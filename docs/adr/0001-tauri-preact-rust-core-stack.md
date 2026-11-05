---
status: accepted
---

# Tauri 2 with a Rust core and a Preact view

Lollipop stays on Tauri 2, with all logic in Rust and a Preact view that only draws state it is given. A separate click-through window is the only mechanism inside the map's data boundary, and it works the same on Tauri, Electron and native, so the choice rests on cost. Tauri already draws over League in Fullscreen with one raw `setLevel` call, ships far less than Electron, and does not need two UI codebases as native does.

## Decisions

- **Window:** one transparent click-through window covering the display. Every Widget lives in it, placed by CSS. On macOS its level is `CGShieldingWindowLevel() + 1`, because League's Fullscreen windows sit at the shield level.
- **Distribution:** Developer ID, not the Mac App Store. Tauri's transparent window needs `macOSPrivateApi`.
- **Logic:** Marker, Benchmark and Timing hint logic is in Rust, so accuracy does not depend on the webview engine, which differs between macOS and Windows.
- **View:** Preact, TypeScript and Vite, with pnpm. Components reach Tauri only through one bridge module.
- **Repository:** a Cargo workspace of `core` (domain, API polling, table format, no Tauri), `app` (Tauri shell) and `collector` (Benchmark CLI).
- **OS-specific code:** one `platform` module in `app`, the only place `unsafe` is allowed.
- **Linters:** strict and blocking on every pull request, with complexity limits in Rust and TypeScript. An exception must state its reason. Nothing blocks on GitHub for now, see [ADR 0009](0009-ci-is-off-while-the-gate-runs-locally.md).
- **Tests:** integration tests through an external boundary, with no unit tests. Only external processes (League, Riot API) are faked. Three levels: `core` against a local HTTP server replaying recorded games, the built app driven through WebdriverIO, and a manual checklist in a real match on each release, run once it is out ([ADR 0021](0021-releases-publish-at-once.md)).

## Considered options

- **Electron:** its `visibleOnFullScreen` was the main draw, but its highest named level, `screen-saver`, is 1000 against a shield level of 2147483628, so it probably needs a raw call too (inferred, not run). It also bundles Chromium: 244-323 MB against 9-57 MB in third-party measurements.
- **Native (AppKit, Win32):** likely the lowest in-game cost, at the price of two UIs and a rewrite.
- **Overwolf:** no macOS overlays, and its overlay is injected, which is off the map.
- **Vanilla JS:** no toolchain, but future Widgets would have no component structure.
- **One window per Widget:** a smaller overlaid area, but one webview and one `setLevel` per window.

## Consequences

- **FPS is unmeasured.** Measure it on the owner's Mac in the first build. If average FPS falls by more than 3% or frame drops are noticeable, first shrink the window to the Widgets' area, then reconsider native. The 3% is a starting estimate.
- **Windows is unproven but not closed off.** Tauri exposes click-through and topmost there, and WebView2 ships with Windows 11. League's present mode and how Vanguard treats a topmost window are unknown. If League's fullscreen is exclusive, players need windowed mode. These risks apply to every separate-window stack, so they are checked on PBE when Windows work starts, not before this decision.
- **A full-display window may affect the game's present path on Windows more than a small one** (inferred). The same FPS check applies there.
- **WebdriverIO on macOS has not been tried.** The fallback is Playwright WebKit against the built UI with injected events, which does not cover the bridge between Rust and the webview.
- **Integration tests locate failures less precisely than unit tests.** Replays run on `gameTime` rather than real time, so `core` needs a controllable clock.
- **The full-display window draws over every app** at this level, so the app must hide it when League is not frontmost.
