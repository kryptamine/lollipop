#!/usr/bin/env node
// Benchmark pilot crawler: fetches ranked solo matches with timelines, an equal
// quota per seed tier. Resumable. PUUIDs stay in memory and never reach disk.
import { appendFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

const args = Object.fromEntries(process.argv.slice(2).map((a) => a.replace(/^--/, '').split('=')));
const TARGET = Number(args.matches ?? 3000);
const LOOKUP_MATCHES_PER_TIER = Number(args.lookups ?? 12);
const OUT = args.out ?? '.probe-logs/benchmark-pilot';
const PLATFORM = 'euw1';
const REGION = 'europe';
const QUEUE = 420;
const MATCHES_PER_SEED = 3;
const LOOKBACK_DAYS = 35;

const TIERS = ['IRON', 'BRONZE', 'SILVER', 'GOLD', 'PLATINUM', 'EMERALD', 'DIAMOND', 'MASTER+'];
const DIVISIONS = ['I', 'II', 'III', 'IV'];
const IDENTITY_KEYS = new Set(['puuid', 'summonerId', 'summonerName', 'riotIdGameName', 'riotIdTagline']);

const KEY = readFileSync(join(homedir(), '.config/lollipop/riot-key'), 'utf8').trim();
mkdirSync(join(OUT, 'samples'), { recursive: true });
const matchesPath = join(OUT, 'matches.ndjson');
const statsPath = join(OUT, 'collect-stats.json');

const stats = { calls: 0, status: {}, rejected: {}, missingTimeline: 0, http429: [], limits: {}, endOfGameResult: {} };
const seen = new Set();
const accepted = Object.fromEntries(TIERS.map((t) => [t, 0]));
const lookedUp = Object.fromEntries(TIERS.map((t) => [t, 0]));
if (existsSync(matchesPath)) {
  for (const line of readFileSync(matchesPath, 'utf8').split('\n')) {
    if (!line) continue;
    const m = JSON.parse(line);
    seen.add(m.matchId);
    accepted[m.seedTier]++;
    if (m.participants.some((p) => p.lookupTier !== undefined)) lookedUp[m.seedTier]++;
  }
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const windows = new Map();

// Sliding windows parsed from Riot's "count:seconds,count:seconds" headers,
// held at 90% so a clock skew against Riot's own window never earns a 429.
async function throttle(scope) {
  for (;;) {
    const w = windows.get(scope);
    if (!w) return;
    const now = Date.now();
    let waitUntil = 0;
    for (const { max, ms, hits } of w) {
      while (hits.length && hits[0] <= now - ms) hits.shift();
      if (hits.length >= max) waitUntil = Math.max(waitUntil, hits[0] + ms);
    }
    if (!waitUntil) return;
    await sleep(waitUntil - now + 5);
  }
}

function learn(scope, header) {
  if (!header) return;
  if (stats.limits[scope] !== header) {
    stats.limits[scope] = header;
    const old = windows.get(scope) ?? [];
    windows.set(scope, header.split(',').map((part) => {
      const [count, seconds] = part.split(':').map(Number);
      const ms = seconds * 1000;
      return { max: Math.floor(count * 0.9), ms, hits: old.find((o) => o.ms === ms)?.hits ?? [] };
    }));
  }
}

const mark = (scope) => { for (const w of windows.get(scope) ?? []) w.hits.push(Date.now()); };

async function riot(host, path, method) {
  for (let attempt = 0; ; attempt++) {
    await throttle('app');
    await throttle(method);
    mark('app');
    mark(method);
    stats.calls++;
    let res;
    try {
      res = await fetch(`https://${host}.api.riotgames.com${path}`, { headers: { 'X-Riot-Token': KEY } });
    } catch {
      if (attempt >= 5) throw new Error(`network failure on ${method}`);
      await sleep(2000 * (attempt + 1));
      continue;
    }
    stats.status[res.status] = (stats.status[res.status] ?? 0) + 1;
    learn('app', res.headers.get('x-app-rate-limit'));
    learn(method, res.headers.get('x-method-rate-limit'));
    if (res.status === 200) return res.json();
    if (res.status === 404) return null;
    if (res.status === 401 || res.status === 403) {
      throw new Error(`key rejected with ${res.status} on ${method}: expired or invalid`);
    }
    if (res.status === 429) {
      const retry = Number(res.headers.get('retry-after') ?? 10);
      stats.http429.push({ method, type: res.headers.get('x-rate-limit-type'), retry });
      await sleep((retry + 1) * 1000);
      continue;
    }
    if (attempt >= 5) throw new Error(`${res.status} on ${method}`);
    await sleep(2000 * (attempt + 1));
  }
}

const pick = (list) => list[Math.floor(Math.random() * list.length)];
function shuffled(list) {
  const out = [...list];
  for (let i = out.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [out[i], out[j]] = [out[j], out[i]];
  }
  return out;
}

async function apexPool() {
  const pool = [];
  for (const league of ['challengerleagues', 'grandmasterleagues', 'masterleagues']) {
    const body = await riot(PLATFORM, `/lol/league/v4/${league}/by-queue/RANKED_SOLO_5x5`, `league-v4:${league}`);
    pool.push(...(body?.entries ?? []));
  }
  return pool;
}

// Pages are sorted by LP, so a random page avoids sampling only the division's top.
async function seedPage(tier, apex) {
  if (tier === 'MASTER+') return apex;
  const division = pick(DIVISIONS);
  let maxPage = 40;
  for (;;) {
    const page = 1 + Math.floor(Math.random() * maxPage);
    const path = `/lol/league/v4/entries/RANKED_SOLO_5x5/${tier}/${division}?page=${page}`;
    const body = await riot(PLATFORM, path, 'league-v4:entries');
    if (body?.length) return body;
    if (maxPage === 1) return [];
    maxPage = Math.max(1, Math.floor(maxPage / 2));
  }
}

// Removes identity fields and the timeline's list of participant PUUIDs.
function scrub(value) {
  if (Array.isArray(value)) return value.map(scrub);
  if (value && typeof value === 'object') {
    const out = {};
    for (const [k, v] of Object.entries(value)) {
      if (IDENTITY_KEYS.has(k)) continue;
      if (k === 'metadata') {
        const { participants: _dropped, ...rest } = v;
        out[k] = rest;
        continue;
      }
      out[k] = scrub(v);
    }
    return out;
  }
  return value;
}

function compact(matchId, seedTier, seedPid, match, timeline, tiers) {
  const info = match.info;
  return {
    matchId,
    seedTier,
    seedPid,
    patch: info.gameVersion.split('.').slice(0, 2).join('.'),
    gameDuration: info.gameDuration,
    endOfGameResult: info.endOfGameResult,
    participants: info.participants.map((p, i) => ({
      pid: p.participantId,
      champion: p.championName,
      role: p.teamPosition,
      individualPosition: p.individualPosition,
      win: p.win,
      goldEarned: p.goldEarned,
      goldSpent: p.goldSpent,
      cs: p.totalMinionsKilled + p.neutralMinionsKilled,
      level: p.champLevel,
      ...(tiers ? { lookupTier: tiers[i] } : {}),
    })),
    frameInterval: timeline.info.frameInterval,
    frames: timeline.info.frames.map((f) => ({
      t: f.timestamp,
      p: Object.fromEntries(Object.entries(f.participantFrames).map(([id, v]) => [
        id,
        [v.minionsKilled, v.jungleMinionsKilled, v.totalGold, v.currentGold, v.level],
      ])),
      items: f.events
        .filter((e) => e.type.startsWith('ITEM_'))
        .map((e) => [e.type, e.participantId, e.itemId ?? e.afterId, e.beforeId]),
    })),
  };
}

async function latestPatches() {
  const res = await fetch('https://ddragon.leagueoflegends.com/api/versions.json');
  const patches = [...new Set((await res.json()).map((v) => v.split('.').slice(0, 2).join('.')))];
  return patches.slice(0, 2);
}

const saveStats = () => writeFileSync(statsPath, JSON.stringify(stats, null, 2));

async function main() {
  const window = await latestPatches();
  stats.patchWindow = window;
  const apex = await apexPool();
  const startTime = Math.floor(Date.now() / 1000) - LOOKBACK_DAYS * 86400;
  const quota = Math.ceil(TARGET / TIERS.length);
  let samplesWritten = 0;

  for (const tier of shuffled(TIERS)) {
    while (accepted[tier] < quota) {
      const page = await seedPage(tier, apex);
      if (!page.length) break;
      for (const seed of shuffled(page).slice(0, tier === 'MASTER+' ? 25 : 10)) {
        if (accepted[tier] >= quota) break;
        if (!seed.puuid) { stats.rejected.seedWithoutPuuid = (stats.rejected.seedWithoutPuuid ?? 0) + 1; continue; }
        const ids = await riot(
          REGION,
          `/lol/match/v5/matches/by-puuid/${seed.puuid}/ids?queue=${QUEUE}&type=ranked&startTime=${startTime}&count=20`,
          'match-v5:ids',
        );
        for (const matchId of shuffled(ids ?? []).filter((id) => !seen.has(id)).slice(0, MATCHES_PER_SEED)) {
          if (accepted[tier] >= quota) break;
          seen.add(matchId);
          const match = await riot(REGION, `/lol/match/v5/matches/${matchId}`, 'match-v5:match');
          if (!match) continue;
          const info = match.info;
          stats.endOfGameResult[info.endOfGameResult] = (stats.endOfGameResult[info.endOfGameResult] ?? 0) + 1;
          const patch = info.gameVersion.split('.').slice(0, 2).join('.');
          const earlySurrender = info.participants.some((p) => p.gameEndedInEarlySurrender);
          const reason = info.queueId !== QUEUE ? 'queue'
            : !window.includes(patch) ? 'patch'
            : info.endOfGameResult !== 'GameComplete' ? 'endOfGameResult'
            : earlySurrender ? 'earlySurrender'
            : null;
          if (reason) { stats.rejected[reason] = (stats.rejected[reason] ?? 0) + 1; continue; }

          const timeline = await riot(REGION, `/lol/match/v5/matches/${matchId}/timeline`, 'match-v5:timeline');
          if (!timeline) { stats.missingTimeline++; continue; }
          const seedPid = info.participants.find((p) => p.puuid === seed.puuid)?.participantId ?? null;

          let tiers = null;
          if (lookedUp[tier] < LOOKUP_MATCHES_PER_TIER) {
            tiers = [];
            for (const p of info.participants) {
              const entries = await riot(PLATFORM, `/lol/league/v4/entries/by-puuid/${p.puuid}`, 'league-v4:by-puuid');
              const solo = (entries ?? []).find((e) => e.queueType === 'RANKED_SOLO_5x5');
              tiers.push(solo?.tier ?? null);
            }
            lookedUp[tier]++;
          }
          if (samplesWritten < 3 && !existsSync(join(OUT, 'samples', `${matchId}.json`))) {
            writeFileSync(join(OUT, 'samples', `${matchId}.json`), JSON.stringify(scrub({ match, timeline })));
            samplesWritten++;
          }
          appendFileSync(matchesPath, `${JSON.stringify(compact(matchId, tier, seedPid, match, timeline, tiers))}\n`);
          accepted[tier]++;
          const total = Object.values(accepted).reduce((a, b) => a + b, 0);
          if (total % 25 === 0) {
            saveStats();
            console.log(`${new Date().toISOString()} ${total} matches, ${stats.calls} calls, ${stats.http429.length} 429s`);
          }
        }
      }
    }
  }
  saveStats();
  console.log(`done: ${JSON.stringify(accepted)}, ${stats.calls} calls`);
}

main().catch((err) => {
  saveStats();
  console.error(err.message);
  process.exit(1);
});
