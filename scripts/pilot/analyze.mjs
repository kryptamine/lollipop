#!/usr/bin/env node
// Turns the crawler's matches.ndjson into the pilot's aggregates, as JSON on stdout.
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const dir = process.argv[2] ?? '.probe-logs/benchmark-pilot';
const matches = readFileSync(join(dir, 'matches.ndjson'), 'utf8').split('\n').filter(Boolean).map((l) => JSON.parse(l));

const MINUTES = [5, 10, 15, 20, 25, 30, 35];
const METRICS = ['cs', 'gold', 'level'];
const ZONES = { cs: 8, gold: 150, level: 1 };
const LADDER = ['IRON', 'BRONZE', 'SILVER', 'GOLD', 'PLATINUM', 'EMERALD', 'DIAMOND', 'MASTER+'];
const MIN_CELL = 5;
const MEDIAN_FACTOR = 1.57;

const tierIndex = (tier) => (['MASTER', 'GRANDMASTER', 'CHALLENGER'].includes(tier) ? 7 : LADDER.indexOf(tier));
const sorted = (xs) => [...xs].sort((a, b) => a - b);
const quantile = (xs, q) => {
  if (!xs.length) return null;
  const s = sorted(xs);
  const at = (s.length - 1) * q;
  const lo = Math.floor(at);
  return s[lo] + (s[Math.min(lo + 1, s.length - 1)] - s[lo]) * (at - lo);
};
const mean = (xs) => xs.reduce((a, b) => a + b, 0) / xs.length;
const variance = (xs) => {
  const m = mean(xs);
  return xs.reduce((a, b) => a + (b - m) ** 2, 0) / (xs.length - 1);
};
const round = (x, d = 3) => (x == null || Number.isNaN(x) ? null : Number(x.toFixed(d)));
const summary = (xs) => ({
  n: xs.length,
  min: xs.length ? Math.min(...xs) : null,
  p5: quantile(xs, 0.05),
  median: quantile(xs, 0.5),
  p95: quantile(xs, 0.95),
  max: xs.length ? Math.max(...xs) : null,
});
const groupBy = (xs, key) => {
  const out = new Map();
  for (const x of xs) {
    const k = key(x);
    if (!out.has(k)) out.set(k, []);
    out.get(k).push(x);
  }
  return out;
};

// Linear interpolation between frames; null once the game has ended.
function at(frames, pid, index, minute) {
  const t = minute * 60000;
  for (let i = 1; i < frames.length; i++) {
    const a = frames[i - 1];
    const b = frames[i];
    if (a.t <= t && t <= b.t) {
      const va = a.p[pid][index];
      const vb = b.p[pid][index];
      return va + ((vb - va) * (t - a.t)) / (b.t - a.t);
    }
  }
  return null;
}

function metricAt(frames, pid, metric, minute) {
  if (metric === 'gold') return at(frames, pid, 2, minute);
  if (metric === 'level') return at(frames, pid, 4, minute);
  const minions = at(frames, pid, 0, minute);
  return minions === null ? null : minions + at(frames, pid, 1, minute);
}

const rows = [];
for (const m of matches) {
  for (const p of m.participants) {
    if (!p.role) continue;
    const values = {};
    for (const metric of METRICS) {
      values[metric] = Object.fromEntries(MINUTES.map((min) => [min, metricAt(m.frames, String(p.pid), metric, min)]));
    }
    rows.push({ cell: `${p.champion}|${p.role}|${m.seedTier}`, role: p.role, seedTier: m.seedTier, values, p, seedPid: m.seedPid });
  }
}

// Spread inside Cells, pooled: a pilot this small has too few games per Cell for
// a per-Cell CV. Zone shares use a leave-one-out median, or a game would count
// towards its own Benchmark and inflate the neutral share.
function spread(metric, minute, filter) {
  const cells = groupBy(rows.filter((r) => filter(r) && r.values[metric][minute] !== null), (r) => r.cell);
  let squares = 0;
  let dof = 0;
  let sum = 0;
  let count = 0;
  let within = 0;
  let above = 0;
  let below = 0;
  let total = 0;
  let cellsUsed = 0;
  for (const cellRows of cells.values()) {
    if (cellRows.length < MIN_CELL) continue;
    cellsUsed++;
    const xs = cellRows.map((r) => r.values[metric][minute]);
    squares += variance(xs) * (xs.length - 1);
    dof += xs.length - 1;
    sum += xs.reduce((a, b) => a + b, 0);
    count += xs.length;
    xs.forEach((x, i) => {
      const med = quantile(xs.filter((_, j) => j !== i), 0.5);
      total++;
      if (Math.abs(x - med) <= ZONES[metric]) within++;
      else if (x > med) above++;
      else below++;
    });
  }
  if (!dof) return null;
  const sd = Math.sqrt(squares / dof);
  const cv = sd / (sum / count);
  const need = (error) => Math.ceil(MEDIAN_FACTOR * ((1.96 * cv) / error) ** 2);
  const halfWidth = (n) => (1.96 * Math.sqrt(MEDIAN_FACTOR) * sd) / Math.sqrt(n);
  return {
    cellsUsed,
    games: total,
    pooledCv: round(cv),
    pooledSd: round(sd, 1),
    medianErrorAt100: round(halfWidth(100), 1),
    medianErrorAt300: round(halfWidth(300), 1),
    nFor5pct: need(0.05),
    nFor3pct: need(0.03),
    nForZoneWidth: Math.ceil(MEDIAN_FACTOR * ((1.96 * sd) / ZONES[metric]) ** 2),
    shareInNeutralZone: round(within / total),
    shareAbove: round(above / total),
    shareBelow: round(below / total),
  };
}

const roles = [...new Set(rows.map((r) => r.role))].sort();
const spreadTable = {};
for (const metric of METRICS) {
  spreadTable[metric] = {};
  for (const minute of MINUTES) {
    const entry = { ALL: spread(metric, minute, () => true) };
    for (const role of roles) entry[role] = spread(metric, minute, (r) => r.role === role);
    spreadTable[metric][minute] = entry;
  }
}

function fieldChecks() {
  const earnedDiff = [];
  const earnedRatioStart = [];
  const unspentDiff = [];
  const csDiff = [];
  const levelDiff = [];
  const frame0Total = [];
  const frame0Current = [];
  const offsets = groupBy([], () => 0);
  const lastGap = [];
  const lastVsDuration = [];
  const intervals = {};
  let totalDrops = 0;
  let csDrops = 0;
  let currentOverTotal = 0;
  let frames = 0;
  let extraIntervalFrames = 0;
  for (const m of matches) {
    intervals[m.frameInterval] = (intervals[m.frameInterval] ?? 0) + 1;
    const last = m.frames.at(-1);
    lastVsDuration.push(last.t - m.gameDuration * 1000);
    lastGap.push(last.t - m.frames.at(-2).t);
    m.frames.forEach((f, i) => {
      if (i < m.frames.length - 1) {
        if (!offsets.has(i)) offsets.set(i, []);
        offsets.get(i).push(f.t - i * 60000);
        if (i > 0 && Math.abs(f.t - m.frames[i - 1].t - 60000) > 1000) extraIntervalFrames++;
      }
    });
    for (const p of m.participants) {
      const id = String(p.pid);
      const [minions, jungle, total, current, level] = last.p[id];
      earnedDiff.push(total - p.goldEarned);
      earnedRatioStart.push(total - p.goldEarned - 500);
      unspentDiff.push(current - (total - p.goldSpent));
      csDiff.push(minions + jungle - p.cs);
      levelDiff.push(level - p.level);
      frame0Total.push(m.frames[0].p[id][2]);
      frame0Current.push(m.frames[0].p[id][3]);
      for (let i = 1; i < m.frames.length; i++) {
        const a = m.frames[i - 1].p[id];
        const b = m.frames[i].p[id];
        if (b[2] < a[2]) totalDrops++;
        if (b[0] + b[1] < a[0] + a[1]) csDrops++;
        if (b[3] > b[2]) currentOverTotal++;
        frames++;
      }
    }
  }
  return {
    frameIntervalValues: intervals,
    frame0TotalGold: summary(frame0Total),
    frame0CurrentGold: summary(frame0Current),
    finalTotalGoldMinusGoldEarned: summary(earnedDiff),
    shareFinalTotalEqualsEarned: round(earnedDiff.filter((d) => d === 0).length / earnedDiff.length),
    shareFinalTotalEqualsEarnedPlus500: round(earnedRatioStart.filter((d) => d === 0).length / earnedRatioStart.length),
    finalCurrentMinusTotalPlusSpent: summary(unspentDiff),
    finalCsMinusMatchCs: summary(csDiff),
    finalLevelMinusChampLevel: summary(levelDiff),
    totalGoldDrops: totalDrops,
    csDrops,
    currentGoldAboveTotal: currentOverTotal,
    participantFramesChecked: frames,
    frameOffsetMsByIndex: Object.fromEntries(
      [...offsets.entries()].filter(([i]) => [1, 5, 10, 15, 20, 25, 30, 35].includes(i)).map(([i, xs]) => [i, summary(xs)]),
    ),
    framesWithIrregularGap: extraIntervalFrames,
    finalFrameGapMs: summary(lastGap),
    finalFrameMinusGameDurationMs: summary(lastVsDuration),
  };
}

function tierLookup() {
  const sampled = matches.filter((m) => m.participants.some((p) => p.lookupTier !== undefined));
  const perTier = {};
  const distances = {};
  let unranked = 0;
  const others = [];
  for (const m of sampled) {
    const seed = LADDER.indexOf(m.seedTier);
    for (const p of m.participants) {
      if (p.pid === m.seedPid) continue;
      if (p.lookupTier === null) { unranked++; continue; }
      const distance = tierIndex(p.lookupTier) - seed;
      distances[distance] = (distances[distance] ?? 0) + 1;
      others.push({ m, p, distance });
      const bucket = (perTier[m.seedTier] ??= { same: 0, n: 0 });
      bucket.n++;
      if (distance === 0) bucket.same++;
    }
  }
  const ladderMedian = (role, tier, metric, minute) => quantile(
    rows.filter((r) => r.role === role && r.seedTier === tier && r.values[metric][minute] !== null)
      .map((r) => r.values[metric][minute]),
    0.5,
  );
  const effect = {};
  for (const metric of METRICS) {
    effect[metric] = {};
    for (const minute of [10, 15, 20]) {
      const errors = [];
      for (const { m, p } of others) {
        if (!p.role) continue;
        const actual = LADDER[tierIndex(p.lookupTier)];
        const a = ladderMedian(p.role, m.seedTier, metric, minute);
        const b = ladderMedian(p.role, actual, metric, minute);
        if (a !== null && b !== null) errors.push(a - b);
      }
      effect[metric][minute] = {
        n: errors.length,
        meanSigned: round(mean(errors), 2),
        meanAbsolute: round(mean(errors.map(Math.abs)), 2),
        p95Absolute: round(quantile(errors.map(Math.abs), 0.95), 2),
      };
    }
  }
  return {
    matchesSampled: sampled.length,
    otherParticipantsRanked: others.length,
    otherParticipantsUnrankedInSolo: unranked,
    tierDistanceCounts: distances,
    sameTierShare: round(others.filter((o) => o.distance === 0).length / others.length),
    withinOneTierShare: round(others.filter((o) => Math.abs(o.distance) <= 1).length / others.length),
    sameTierShareBySeedTier: Object.fromEntries(Object.entries(perTier).map(([t, b]) => [t, round(b.same / b.n)])),
    benchmarkShiftFromMislabel: effect,
  };
}

function ladder() {
  const out = {};
  for (const role of roles) {
    out[role] = {};
    for (const tier of LADDER) {
      const subset = rows.filter((r) => r.role === role && r.seedTier === tier);
      out[role][tier] = {
        cs15: round(quantile(subset.map((r) => r.values.cs[15]).filter((x) => x !== null), 0.5), 1),
        gold15: round(quantile(subset.map((r) => r.values.gold[15]).filter((x) => x !== null), 0.5), 0),
      };
    }
  }
  return out;
}

const cellSizes = [...groupBy(rows, (r) => r.cell).values()].map((c) => c.length);
console.log(JSON.stringify({
  matches: matches.length,
  matchesBySeedTier: Object.fromEntries([...groupBy(matches, (m) => m.seedTier)].map(([k, v]) => [k, v.length])),
  matchesByPatch: Object.fromEntries([...groupBy(matches, (m) => m.patch)].map(([k, v]) => [k, v.length])),
  gameDurationMinutes: summary(matches.map((m) => m.gameDuration / 60)),
  gamesUnder15Minutes: matches.filter((m) => m.gameDuration < 900).length,
  gamesReachingMinute: Object.fromEntries(MINUTES.map((min) => [min, round(matches.filter((m) => m.frames.at(-1).t >= min * 60000).length / matches.length)])),
  participantsWithoutRole: matches.flatMap((m) => m.participants).filter((p) => !p.role).length,
  roleDisagreesWithIndividualPosition: round(rows.filter((r) => r.p.individualPosition !== r.role).length / rows.length),
  distinctChampionRolePairs: new Set(rows.map((r) => r.cell.split('|').slice(0, 2).join('|'))).size,
  cells: { distinct: cellSizes.length, atLeast15: cellSizes.filter((n) => n >= MIN_CELL).length, largest: Math.max(...cellSizes) },
  fields: fieldChecks(),
  spread: spreadTable,
  tierLookup: tierLookup(),
  ladder: ladder(),
}, null, 2));
