// Fake Live Client Data API and League client for developing without a running game.
// Usage: make dev-mock
import { writeFileSync } from "node:fs";
import { createServer } from "node:http";

const PORT = 2998;
const [lockfile] = process.argv.slice(2);
const startedAt = Date.now();

const leagueClient = new Map([
  ["/lol-gameflow/v1/session", { gameData: { queue: { id: 420 } } }],
  ["/lol-patch/v1/game-version", "16.21.8342754+branch.releases-16-21.code.public"],
  ["/lol-ranked/v1/current-ranked-stats", { queues: [{ queueType: "RANKED_SOLO_5x5", tier: "GOLD" }] }],
]);

function liveClientData() {
  // Starts at 0:30 game time, early enough for Gold earned to count.
  const gameTime = 30 + (Date.now() - startedAt) / 1000;
  const minutes = gameTime / 60;
  return {
    activePlayer: { riotId: "Me#DEV", currentGold: 500 + gameTime * 3 },
    allPlayers: [
      {
        riotId: "Me#DEV",
        championName: "Ahri",
        position: "MIDDLE",
        level: Math.min(18, 1 + Math.floor(minutes)),
        items: [],
        // The API floors CS to 10.
        scores: { creepScore: Math.floor(minutes * 0.6) * 10 },
      },
    ],
    events: { Events: [{ EventName: "GameStart" }] },
    gameData: { gameTime, gameMode: "CLASSIC" },
  };
}

writeFileSync(lockfile, `LeagueClient:0:${PORT}:mock:http`);

createServer((req, res) => {
  res.setHeader("content-type", "application/json");
  res.end(JSON.stringify(leagueClient.get(req.url) ?? liveClientData()));
}).listen(PORT, "127.0.0.1", () => console.log(`mock live client on http://127.0.0.1:${PORT}/`));
