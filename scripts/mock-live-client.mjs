// Fake Live Client Data API for developing without a running game.
// Usage: make dev-mock
import { createServer } from "node:http";

const startedAt = Date.now();

createServer((_req, res) => {
  // Starts at 0:30 game time, early enough for Gold earned to count.
  const gameTime = 30 + (Date.now() - startedAt) / 1000;
  const minutes = gameTime / 60;
  res.setHeader("content-type", "application/json");
  res.end(
    JSON.stringify({
      activePlayer: { riotId: "Me#DEV", currentGold: 500 + gameTime * 3 },
      allPlayers: [
        {
          riotId: "Me#DEV",
          championName: "Ahri",
          level: Math.min(18, 1 + Math.floor(minutes)),
          items: [],
          // The API floors CS to 10.
          scores: { creepScore: Math.floor(minutes * 0.6) * 10 },
        },
      ],
      events: { Events: [{ EventName: "GameStart" }] },
      gameData: { gameTime, gameMode: "CLASSIC" },
    }),
  );
}).listen(2998, "127.0.0.1", () => console.log("mock live client on http://127.0.0.1:2998/"));
