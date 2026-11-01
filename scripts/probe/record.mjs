// Throwaway recorder for the live-game probe: logs changes only, as NDJSON.
// Player names are replaced, and the LCU password never leaves memory.
// Usage: node scripts/probe/record.mjs   (type a line + Enter to log a note)
import { execFileSync } from "node:child_process";
import { appendFileSync, existsSync, mkdirSync, readFileSync } from "node:fs";
import http from "node:http";
import https from "node:https";
import { createInterface } from "node:readline";

const LIVE_BASE = process.env.PROBE_LIVE_BASE ?? "https://127.0.0.1:2999";
const LIVE_POLL_MS = 250;
const LCU_POLL_MS = 5000;
const DEFAULT_INSTALL_DIR = "/Applications/League of Legends.app/Contents/LoL";

mkdirSync(".probe-logs", { recursive: true });
const logPath = `.probe-logs/record-${new Date().toISOString().replace(/[:.]/g, "-")}.ndjson`;
const log = (kind, data) =>
  appendFileSync(logPath, `${JSON.stringify({ at: new Date().toISOString(), kind, ...data })}\n`);

// Both local APIs serve loopback HTTPS with a self-signed certificate.
const insecureAgent = new https.Agent({ rejectUnauthorized: false });

function get(url, headers = {}) {
  const lib = url.startsWith("https") ? https : http;
  return new Promise((resolve) => {
    const req = lib.get(url, { agent: url.startsWith("https") ? insecureAgent : undefined, headers, timeout: 1000 }, (res) => {
      let body = "";
      res.on("data", (chunk) => (body += chunk));
      res.on("end", () => resolve({ status: res.statusCode, body }));
    });
    req.on("timeout", () => req.destroy(Object.assign(new Error("timeout"), { code: "TIMEOUT" })));
    req.on("error", (err) => resolve({ status: err.code ?? "ERROR", body: "" }));
  });
}

function parse(body) {
  try {
    return JSON.parse(body);
  } catch {
    return undefined;
  }
}

function keyPaths(value, prefix = "", out = new Set()) {
  if (Array.isArray(value)) value.forEach((item) => keyPaths(item, `${prefix}[]`, out));
  else if (value && typeof value === "object")
    for (const [key, item] of Object.entries(value)) keyPaths(item, prefix ? `${prefix}.${key}` : key, out);
  else out.add(prefix);
  return out;
}

function nameMap(data, self) {
  const names = new Map();
  data.allPlayers.forEach((player, index) => {
    const label = player === self ? "self" : `player-${index}`;
    for (const key of ["riotId", "riotIdGameName", "summonerName"]) if (player[key]) names.set(player[key], label);
    if (player.riotIdTagLine) names.set(player.riotIdTagLine, "tag");
  });
  const active = data.activePlayer ?? {};
  for (const key of ["riotId", "riotIdGameName", "summonerName"]) if (active[key]) names.set(active[key], "self");
  if (active.riotIdTagLine) names.set(active.riotIdTagLine, "tag");
  return names;
}

function redact(value, names) {
  if (typeof value === "string") return names.get(value) ?? value;
  if (Array.isArray(value)) return value.map((item) => redact(item, names));
  if (value && typeof value === "object")
    return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, redact(item, names)]));
  return value;
}

function findSelf(data) {
  const active = data.activePlayer ?? {};
  return data.allPlayers?.find(
    (player) =>
      (active.riotId && player.riotId === active.riotId) ||
      (active.summonerName && player.summonerName === active.summonerName),
  );
}

let liveStatus;
let lastState;
let lastGameTime = -1;
let seenEvents = new Set();
let firstPayloadLogged = false;

async function logFirstPayload(data, self, names) {
  firstPayloadLogged = true;
  const { allPlayers, events, ...rest } = data;
  const stats = await get(`${LIVE_BASE}/liveclientdata/gamestats`);
  const openapi = await get(`${LIVE_BASE}/swagger/v3/openapi.json`);
  log("first-payload", {
    payload: redact({ ...rest, self }, names),
    keyPaths: [...keyPaths(data)].sort(),
    gamestats: { status: stats.status, body: parse(stats.body) },
    openapi: { status: openapi.status, paths: Object.keys(parse(openapi.body)?.paths ?? {}) },
  });
}

async function pollLive() {
  const { status, body } = await get(`${LIVE_BASE}/liveclientdata/allgamedata`);
  const data = status === 200 ? parse(body) : undefined;
  const self = data && findSelf(data);
  const nextStatus = status === 200 && !self ? "200-without-active-player" : status;
  if (nextStatus !== liveStatus) {
    liveStatus = nextStatus;
    log("live-status", { status: nextStatus, topLevelKeys: data ? Object.keys(data) : [] });
    console.log(`live: ${nextStatus}`);
  }
  if (!self) return;

  const gameTime = data.gameData?.gameTime ?? 0;
  if (gameTime < lastGameTime) {
    firstPayloadLogged = false;
    seenEvents = new Set();
    lastState = undefined;
    log("game-restarted", { gameTime, lastGameTime });
  }
  lastGameTime = gameTime;

  const names = nameMap(data, self);
  if (!firstPayloadLogged) await logFirstPayload(data, self, names);

  const state = {
    gameMode: data.gameData?.gameMode,
    mapNumber: data.gameData?.mapNumber,
    champion: self.championName,
    position: self.position,
    cs: self.scores?.creepScore,
    gold: data.activePlayer?.currentGold,
    level: data.activePlayer?.level,
    items: self.items,
  };
  const serialized = JSON.stringify(state);
  if (serialized !== lastState) {
    lastState = serialized;
    log("state", { gameTime, ...state });
    const minutes = `${Math.floor(gameTime / 60)}:${String(Math.floor(gameTime % 60)).padStart(2, "0")}`;
    console.log(`${minutes} cs ${state.cs} gold ${Math.floor(state.gold ?? 0)} lvl ${state.level} items ${state.items?.length ?? 0} pos ${state.position}`);
  }

  for (const event of data.events?.Events ?? []) {
    if (seenEvents.has(event.EventID)) continue;
    seenEvents.add(event.EventID);
    log("event", { gameTime, event: redact(event, names) });
    console.log(`event ${event.EventName} at ${event.EventTime}`);
  }
}

function installDirFromProcess() {
  try {
    const args = execFileSync("ps", ["x", "-o", "args"], { encoding: "utf8", maxBuffer: 1 << 24 });
    const line = args.split("\n").find((entry) => entry.includes("LeagueClientUx") && entry.includes("--install-directory="));
    return line?.match(/--install-directory=(.*?)(?: --|$)/)?.[1];
  } catch {
    return undefined;
  }
}

let lcuStatus;
let lcuPhase;
let lastChampSelect;
let lcuBaseLogged = false;

async function pollLcu() {
  const fromProcess = installDirFromProcess();
  const lockfilePath = `${fromProcess ?? DEFAULT_INSTALL_DIR}/lockfile`;
  const lockfile = existsSync(lockfilePath) ? readFileSync(lockfilePath, "utf8").trim().split(":") : undefined;
  const [, , port, password, protocol] = lockfile ?? [];
  const request = (path) =>
    get(`${protocol}://127.0.0.1:${port}${path}`, {
      authorization: `Basic ${Buffer.from(`riot:${password}`).toString("base64")}`,
    });

  const phaseResponse = lockfile ? await request("/lol-gameflow/v1/gameflow-phase") : { status: "NO_LOCKFILE", body: "" };
  const nextStatus = JSON.stringify({ lockfilePath, installDirFromProcess: Boolean(fromProcess), status: phaseResponse.status });
  if (nextStatus !== lcuStatus) {
    lcuStatus = nextStatus;
    log("lcu-status", JSON.parse(nextStatus));
    console.log(`lcu: ${phaseResponse.status} (${lockfilePath})`);
  }
  if (phaseResponse.status !== 200) {
    lcuBaseLogged = false;
    return;
  }

  if (!lcuBaseLogged) {
    lcuBaseLogged = true;
    const summoner = await request("/lol-summoner/v1/current-summoner");
    const version = await request("/lol-patch/v1/game-version");
    const ranked = await request("/lol-ranked/v1/current-ranked-stats");
    const queues = parse(ranked.body)?.queues ?? [];
    log("lcu-base", {
      currentSummonerStatus: summoner.status,
      gameVersion: { status: version.status, body: parse(version.body) },
      ranked: {
        status: ranked.status,
        keyPaths: [...keyPaths(parse(ranked.body) ?? {})].sort(),
        queues: queues.map(({ queueType, tier, division, leaguePoints }) => ({ queueType, tier, division, leaguePoints })),
      },
    });
  }

  const phase = parse(phaseResponse.body);
  if (phase !== lcuPhase) {
    lcuPhase = phase;
    const session = parse((await request("/lol-gameflow/v1/session")).body);
    const queue = session?.gameData?.queue;
    log("lcu-phase", {
      phase,
      queue: queue && { id: queue.id, type: queue.type, gameMode: queue.gameMode, mapId: queue.mapId },
    });
    console.log(`lcu phase: ${phase}`);
  }

  if (phase === "ChampSelect") {
    const champSelect = parse((await request("/lol-champ-select/v1/session")).body);
    const me = champSelect?.myTeam?.find((entry) => entry.cellId === champSelect.localPlayerCellId);
    const summary = JSON.stringify({ assignedPosition: me?.assignedPosition, championId: me?.championId });
    if (summary !== lastChampSelect) {
      lastChampSelect = summary;
      log("lcu-champ-select", JSON.parse(summary));
    }
  }
}

function loop(poll, intervalMs) {
  const tick = async () => {
    try {
      await poll();
    } catch (err) {
      log("recorder-error", { source: poll.name, message: String(err) });
    }
    setTimeout(tick, intervalMs);
  };
  tick();
}

createInterface({ input: process.stdin }).on("line", (text) => {
  log("note", { text, gameTime: lastGameTime });
  console.log(`note logged at ${lastGameTime}`);
});

console.log(`recording to ${logPath}; Ctrl+C to stop`);
loop(pollLive, LIVE_POLL_MS);
loop(pollLcu, LCU_POLL_MS);
