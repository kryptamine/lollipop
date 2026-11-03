// Fake Live Client Data API and League client for developing without a running game.
// Usage: make dev-mock, or MOCK_PATCH=16.24 make dev-mock to show a Stale table.
import { writeFileSync } from "node:fs";
import { createServer } from "node:http";

const PORT = 2998;
const [lockfile, benchmarkTable] = process.argv.slice(2);
const startedAt = Date.now();
const patch = process.env.MOCK_PATCH ?? "16.21";

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
  return {
    schema_version: 2,
    patch_window: ["16.20", "16.21"],
    cells: [{ champion: "Ahri", role: "MIDDLE", tier: "GOLD", minutes }],
  };
}

// The first seconds are the loading screen, which asks for loading screen Advice.
const LOADING_SCREEN_S = 5;

// Stands in for a Responses call: the model's JSON as the output text.
const MOCK_ADVICE = {
  output: [
    {
      type: "message",
      content: [
        {
          type: "output_text",
          text: JSON.stringify({ advice: "Mock Advice: trade when Zed uses W, then back off.", item: null }),
        },
      ],
    },
  ],
};

function liveClientData() {
  // Starts at 0:00 game time after the loading screen, so the notices of the first 10 s show.
  const elapsed = (Date.now() - startedAt) / 1000;
  const loading = elapsed < LOADING_SCREEN_S;
  const gameTime = loading ? 0 : elapsed - LOADING_SCREEN_S;
  const minutes = gameTime / 60;
  return {
    activePlayer: { riotId: "Me#DEV", currentGold: 500 + gameTime * 3 },
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
        items: [],
        // The API floors CS to 10.
        scores: {
          creepScore: Math.floor(minutes * 0.6) * 10,
          kills: Math.floor(minutes / 4),
          deaths: Math.floor(minutes / 8),
          assists: Math.floor(minutes / 3),
        },
      },
    ],
    events: { Events: loading ? [] : [{ EventName: "GameStart", EventTime: 0 }] },
    gameData: { gameTime, gameMode: "CLASSIC" },
  };
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
    req.resume();
    res.end(JSON.stringify(MOCK_ADVICE));
    return;
  }
  res.end(JSON.stringify(leagueClient.get(req.url) ?? liveClientData()));
}).listen(PORT, "127.0.0.1", () => console.log(`mock live client on http://127.0.0.1:${PORT}/`));
