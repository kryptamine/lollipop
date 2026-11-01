// Fake Live Client Data API for developing without a running game.
// Usage: node scripts/mock-live-client.mjs
//        LOLLIPOP_LIVE_URL=http://127.0.0.1:2998/ pnpm tauri dev
import { createServer } from "node:http";

const startedAt = Date.now();

createServer((_req, res) => {
  // Starts at 10:00 game time and gains CS at 6/min.
  const gameTime = 600 + (Date.now() - startedAt) / 1000;
  res.setHeader("content-type", "application/json");
  res.end(
    JSON.stringify({
      activePlayer: { riotId: "Me#DEV" },
      allPlayers: [
        { riotId: "Me#DEV", championName: "Ahri", scores: { creepScore: Math.floor((gameTime / 60) * 6) } },
      ],
      gameData: { gameTime, gameMode: "CLASSIC" },
    }),
  );
}).listen(2998, "127.0.0.1", () => console.log("mock live client on http://127.0.0.1:2998/"));
