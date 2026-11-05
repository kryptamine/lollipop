// Fake Live Client Data API and League client for developing without a running game.
// Usage: make dev-mock, or MOCK_PATCH=16.24 make dev-mock to show a Stale table.
// MOCK_OPENAI=down, rejected or quota makes every Advice call fail that way.
import { writeFileSync } from "node:fs";
import { createServer } from "node:http";

const PORT = 2998;
const [lockfile, benchmarkTable] = process.argv.slice(2);
const startedAt = Date.now();
const patch = process.env.MOCK_PATCH ?? "16.21";
const openaiFailure = process.env.MOCK_OPENAI;

const leagueClient = new Map([
  ["/lol-gameflow/v1/session", { gameData: { queue: { id: 420 } } }],
  ["/lol-patch/v1/game-version", `${patch}.8342754+branch.code.public`],
  ["/lol-ranked/v1/current-ranked-stats", { queues: [{ queueType: "RANKED_SOLO_5x5", tier: "GOLD" }] }],
]);

// The mock player trails on CS, leads on gold and matches on level, to show each Marker.
function mockTable() {
  const minutes = Array.from({ length: 61 }, (_, minute) => ({
    minute,
    cs: minute * 9,
    total_gold: 500 + minute * 120,
    level: Math.min(18, 1 + minute),
    n: 300,
    kills: minute * 0.2,
    deaths: minute * 0.2,
    assists: minute * 0.3,
    scores_n: 300,
  }));
  const candidates = [
    { item: 3020, share: 0.8, minute: 10 },
    { item: 3047, share: 0.3, minute: 9 },
    { item: 3157, share: 0.6, minute: 18 },
    { item: 3102, share: 0.45, minute: 21 },
    { item: 3089, share: 0.4, minute: 25 },
    { item: 3135, share: 0.3, minute: 28 },
  ];
  return {
    schema_version: 2,
    patch_window: ["16.20", "16.21"],
    cells: [{ champion: "Ahri", role: "MIDDLE", tier: "GOLD", minutes }],
    damage_profile: { magic_share: { Zed: 0.1 }, mixed_from: 1, magic_heavy_from: 1.5 },
    item_candidates: [
      { champion: "Ahri", role: "MIDDLE", physical_heavy: candidates, mixed: candidates, magic_heavy: candidates },
    ],
    data_dragon_version: "16.21.1",
    item_names: {
      3020: { en: "Sorcerer's Shoes", ru: "Сапоги чародея" },
      3047: { en: "Plated Steelcaps", ru: "Бронированные сапоги" },
      3157: { en: "Zhonya's Hourglass", ru: "Песочные часы Жони" },
      3102: { en: "Banshee's Veil", ru: "Покров банши" },
      3089: { en: "Rabadon's Deathcap", ru: "Смертельная шляпа Рабадона" },
      3135: { en: "Void Staff", ru: "Посох Бездны" },
    },
    boots: [3006, 3020, 3047],
    completed: [3089, 3102, 3135, 3157],
  };
}

// Zed kills the mock player at this game time, and they respawn RESPAWN_S later.
const DEATH_AT_S = 40;
const RESPAWN_S = 15;

// At this game time the mock player buys Sorcerer's Shoes, which leave the Build.
const BUY_AT_S = 120;
const SHOES = { itemID: 3020, count: 1, displayName: "Sorcerer's Shoes" };
const SHOES_COST = 1100;

// The first seconds are the loading screen, which asks for loading screen Advice.
const LOADING_SCREEN_S = 5;

// Stands in for a Responses call: the model's JSON as the output text.
function mockAdvice(answer) {
  return { output: [{ type: "message", content: [{ type: "output_text", text: JSON.stringify(answer) }] }] };
}

const LANE_PLAN = {
  advice: "Mock Advice: trade when Zed uses W, then back off.",
  next_item: 3047,
  reason: "Mock Build change: Zed and Lee Sin hit physical, so Plated Steelcaps first.",
  swap: null,
  elixir: null,
};
const DEATH_ADVICE = { advice: "Mock Advice: Zed's ult kills you, keep E for when he lands.", next_item: null, reason: null, swap: null, elixir: null };
const TURNING_POINT_ADVICE = { advice: "Mock Advice: level 6, look for a roam to the bottom lane.", next_item: null, reason: null, swap: null, elixir: null };

function liveClientData() {
  // Starts at 0:00 game time after the loading screen, so the notices of the first 10 s show.
  const elapsed = (Date.now() - startedAt) / 1000;
  const loading = elapsed < LOADING_SCREEN_S;
  const gameTime = loading ? 0 : elapsed - LOADING_SCREEN_S;
  const minutes = gameTime / 60;
  const isDead = gameTime >= DEATH_AT_S && gameTime < DEATH_AT_S + RESPAWN_S;
  const hasBought = gameTime >= BUY_AT_S;
  const death = { EventName: "ChampionKill", EventTime: DEATH_AT_S, KillerName: "Rival#DEV", VictimName: "Me#DEV", Assisters: [] };
  const events = loading ? [] : [{ EventName: "GameStart", EventTime: 0 }];
  if (gameTime >= DEATH_AT_S) {
    events.push(death);
  }
  return {
    activePlayer: { riotId: "Me#DEV", currentGold: 500 + gameTime * 3 - (hasBought ? SHOES_COST : 0) },
    allPlayers: [
      {
        riotId: "Rival#DEV",
        championName: "Zed",
        team: "CHAOS",
        position: "MIDDLE",
        level: 1,
        items: [],
        scores: { creepScore: 0 },
      },
      {
        riotId: "Me#DEV",
        championName: "Ahri",
        team: "ORDER",
        position: "MIDDLE",
        level: Math.min(18, 1 + Math.floor(minutes)),
        isDead,
        respawnTimer: isDead ? DEATH_AT_S + RESPAWN_S - gameTime : 0,
        items: hasBought ? [SHOES] : [],
        // The API floors CS to 10.
        scores: {
          creepScore: Math.floor(minutes * 0.6) * 10,
          kills: Math.floor(minutes / 4),
          deaths: Math.floor(minutes / 8),
          assists: Math.floor(minutes / 3),
        },
      },
    ],
    events: { Events: events },
    gameData: { gameTime, gameMode: "CLASSIC" },
  };
}

// The ways an Advice call can fail that the app tells the player about.
function failAdvice(res) {
  const quota = { error: { code: "insufficient_quota", type: "insufficient_quota" } };
  const failures = { down: [503, {}], rejected: [401, {}], quota: [429, quota] };
  const [status, body] = failures[openaiFailure] ?? failures.down;
  res.statusCode = status;
  res.end(JSON.stringify(body));
}

// Tells the Triggers apart by the moment their prompt names.
function adviceFor(prompt) {
  if (prompt.includes("just killed")) {
    return DEATH_ADVICE;
  }
  if (prompt.includes("changed the game plan")) {
    return TURNING_POINT_ADVICE;
  }
  return LANE_PLAN;
}

writeFileSync(lockfile, `LeagueClient:0:${PORT}:mock:http`);
writeFileSync(benchmarkTable, JSON.stringify(mockTable()));

createServer((req, res) => {
  res.setHeader("content-type", "application/json");
  // Stands in for OpenAI's key check: a key starting `sk-mock` is good, any other is rejected.
  if (req.url === "/v1/models/gpt-6-luna") {
    res.statusCode = req.headers.authorization?.startsWith("Bearer sk-mock") ? 200 : 401;
    res.end(JSON.stringify({ id: "gpt-6-luna" }));
    return;
  }
  if (req.method === "POST" && req.url === "/v1/responses") {
    if (openaiFailure !== undefined) {
      req.resume();
      failAdvice(res);
      return;
    }
    let body = "";
    req.on("data", (chunk) => (body += chunk));
    req.on("end", () => {
      res.end(JSON.stringify(mockAdvice(adviceFor(body))));
    });
    return;
  }
  res.end(JSON.stringify(leagueClient.get(req.url) ?? liveClientData()));
}).listen(PORT, "127.0.0.1", () => console.log(`mock live client on http://127.0.0.1:${PORT}/`));
