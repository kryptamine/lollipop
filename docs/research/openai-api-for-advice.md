# OpenAI API for Advice

Research for "OpenAI API for Advice" (#88), part of "Map: AI Advice" (#86).
Every page below was read on 2026-11-02. OpenAI renames models, tiers and
prices often (Priority processing became Fast mode on 2026-07-30), so check
the figures again before an implementation pins them.

Nothing here was measured with a real key. Latency and the Free tier's limits
are still open, see [Open questions](#open-questions).

## Answer

| Question | Answer |
|---|---|
| Model | `gpt-6-luna`, with `gpt-6.1-terra` as the step up if Advice quality is poor |
| API shape | `POST /v1/responses`, `store: false`, `reasoning.effort: "none"`, a strict `json_schema` text format |
| Rejecting items outside the candidates | A fixed schema with an item id string, checked against the candidates in code |
| Cost per call | About $0.0002 on `gpt-6-luna`, about $0.002 on `gpt-6.1-terra` (1,500 tokens in, 80 out) |
| Cost per game | About $0.005 on `gpt-6-luna` at 25 calls |
| Rate limits | Build tier: 5,000 RPM, 2M TPM on `gpt-6-luna`. Advice needs about 1 RPM. Free tier limits are not published |
| Key check on save | `GET /v1/models/gpt-6-luna`: free of tokens, but it misses an empty balance |
| Rust | No official SDK. Plain `reqwest` plus private serde structs, on the stack `core` already has |

## Model

The models page lists two flagships (`gpt-6-astra`, `gpt-6.1-sol`) and two
small models: `gpt-6.1-terra`, "fast, efficient model for visual reasoning,
precise computer use, and clear, natural writing", and `gpt-6-luna`, "most
efficient model, for focused, high-volume tasks". None is marked deprecated.
[1]

| | `gpt-6-luna` | `gpt-6.1-terra` |
|---|---|---|
| Input / cached / output, $ per 1M | 0.10 / 0.01 / 0.50 | 1.00 / 0.05 / 5.00 |
| 30-minute cache write, $ per 1M | 0.125 | 1.25 |
| Context / max output | 1,050,000 / 128,000 | 1,050,000 / 128,000 |
| Knowledge cutoff | 2026-05-18 | 2026-06-01 |
| `reasoning.effort` | `none` to `max`, default `medium` | `none` to `max`, default `medium` |
| Structured outputs, streaming | Yes | Yes |
| Endpoints | Responses, Chat Completions, Batch | Responses, Chat Completions, Batch |

Sources: [2], [3], [4]. Neither model page gives a speed rating or a release
date.

Advice is a sentence or two plus an optional item from a short list, which is
a focused task. `gpt-6-luna` costs a tenth of `gpt-6.1-terra`, so it is the
starting point. The knowledge cutoff matters: neither model knows the current
patch, so the prompt has to carry everything the Advice depends on (items,
their costs, the Benchmark).

Older small models (`gpt-5.4-nano`, `gpt-5-nano`, `gpt-4.1-nano`, `gpt-4o-mini`)
are still priced [4] but are older generations with no reason to prefer them.

## API shape

**Responses, not Chat Completions.** "While Chat Completions remains supported,
Responses is recommended for all new projects." [5] On GPT-6 models, function
calling in Chat Completions works only with reasoning off [2], another sign
it is the second-class path.

**`store: false`.** "Responses are stored by default." [5] A stored response is
kept 30 days as application state [6]. Advice needs no conversation state, so
nothing should be stored. Abuse-monitoring logs are kept up to 30 days
regardless, and API data is not used for training by default [6].

**`reasoning.effort: "none"`.** The default is `medium` on both models [2][3].
Reasoning tokens are billed as output and count against `max_output_tokens`,
and a response cut off there may carry no visible text at all [7]. `none` "turns
off reasoning" and fits tasks such as classification and fast retrieval [7].
The guide suggests `low` first for latency-sensitive work on `gpt-5.5` [7]; for
Advice, start at `none` and raise it only if fixtures show bad picks.

**`max_output_tokens`.** Cap it (a few hundred tokens with reasoning off) and
treat `status: "incomplete"` as a failed call: the output may not match the
schema [8].

**Latency levers.** Output length dominates: "cutting 50% of your output
tokens may cut ~50% of your latency", while halving input buys 1-5% [9]. So
keep the answer short and its field names short. Streaming "cuts the waiting
time to a second or less" [9], but a Widget shows a finished sentence, so it
helps only if Voiceover wants to start early. Fast mode (`service_tier:
"fast"`) promises "up to 2.5× faster speeds and more consistent latency" at 2x
the price [10]; on `gpt-6-luna` that is still $0.20 / $1.00 per 1M [4].

**Prompt caching.** Automatic on GPT-5.6 and later once the shared prefix
reaches 1,024 tokens [11]. Put the fixed instructions first and the game state
last. A 30-minute write costs 1.25x input, a read 0.1x [11]. Over a 30-minute
game with calls every few minutes the cache stays warm, but at Advice's prompt
size the saving is a fraction of a cent per game.

### Rejecting items outside the candidates

Strict structured output: `text: { format: { type: "json_schema", strict: true,
name, schema } }` [8]. Every object needs `additionalProperties: false`, every
field must be `required` (optional means `["string", "null"]`), and the root
must be an object without `anyOf` [8]. A refusal comes back as a content item of
type `refusal` and does not follow the schema [8].

`enum` is supported, up to 1,000 values per schema [8], so the candidates could
go straight into the schema. But "the first request you make with any schema
will have additional latency" [8], and candidates change per champion, per
Trigger and per purchase, so most calls would pay it. A fixed schema avoids
that:

```json
{
  "type": "object",
  "properties": {
    "advice": { "type": "string" },
    "item": { "type": ["integer", "null"] }
  },
  "required": ["advice", "item"],
  "additionalProperties": false
}
```

The candidates go into the prompt, and code drops any `item` not among them,
which the map already requires. Whether the drop throws the whole Advice away
or shows the text without the item is a decision for the failure-behaviour
ticket.

## Cost

Assumed prompt: ~1,000 tokens of fixed instructions plus ~500 of game state
and candidates, ~80 tokens out, reasoning off. Prices from [4], Standard tier.

| | `gpt-6-luna` | `gpt-6.1-terra` |
|---|---|---|
| One call, no cache | $0.00019 | $0.0019 |
| 25 calls (one game) | $0.005 | $0.05 |
| 1,000 games | $5 | $50 |

25 calls is one loading screen, about 6 deaths, 8 backs and 10 periodic calls
in a 30-minute game. Reasoning at `low` adds its tokens at the output rate,
roughly another $0.0001 per call on `gpt-6-luna` for every 200 tokens.

At these figures a cost cap per game is not needed on `gpt-6-luna`. A
misbehaving Trigger loop is the real risk, and a per-game call limit in code
covers it more cheaply than a spend counter.

## Rate limits

Tiers are Free, Build ($5 of credit bought, $500/month usage limit), Launch
($100, $5,000/month) and Grow ($500, $200,000/month). Free requires a supported
country and has a $100/month limit. Upgrades are automatic [12].

| Tier | `gpt-6-luna` RPM / TPM | `gpt-6.1-terra` RPM / TPM |
|---|---|---|
| Build | 5,000 / 2,000,000 | 5,000 / 1,000,000 |
| Launch | 10,000 / 10,000,000 | 10,000 / 4,000,000 |
| Grow | 30,000 / 180,000,000 | 15,000 / 40,000,000 |

Sources: [2], [3]. Advice needs about one request a minute, so any paid tier is
far above it. Neither model page lists Free tier limits [2][3].

Errors a client must tell apart [12][13]:

| Status | `error.code` | Meaning | Retry |
|---|---|---|---|
| 401 | | Invalid, revoked or wrong-project key | No |
| 403 | | Unsupported country | No |
| 429 | (rate limit), `slow_down` | Throttled; honour `Retry-After` | Yes |
| 429 | `credit_balance_exhausted`, `organization_spend_limit_exceeded`, `project_spend_limit_exceeded`, `organization_usage_limit_exceeded` | Billing; `error.type` may be `insufficient_quota` | No |
| 500 | | Server error | Yes |
| 503 | `server_is_overloaded` | Model overloaded | Yes |

A 429 is retryable only by its `error.code`, not by status alone. For in-game
Advice a retry mostly arrives too late, so "no retry, skip this Trigger" is
likely the simplest correct behaviour.

## Validating a key on save

`GET /v1/models/{model}` returns the Model object [14]; `GET /v1/models` lists
them [15]. Neither page states a cost, and neither generates tokens. Calling it
with `gpt-6-luna` checks the key and that the key's project can see the fixed
model in one request.

It does not catch everything:

- An empty credit balance shows only on a billed call (429
  `credit_balance_exhausted`) [13].
- A restricted key needs `api.model.read` for `/v1/models` and the separate
  Responses permission for `/v1/responses` [16]. A key with only one of them
  passes or fails the check wrongly.

The alternative is a real `POST /v1/responses` with a one-word input and a tiny
`max_output_tokens`. It catches all three cases for well under $0.0001, but it
spends the player's money on every save. Suggested: the free model check on
save, and the first real Advice call's error shown as the key's state.

## Rust

OpenAI's official SDKs are JavaScript, Python, .NET, Java, Go and Ruby. Rust
appears only under community libraries, which OpenAI "does not verify" [17].

| Crate | Latest | Updated | Downloads (recent) | Note |
|---|---|---|---|---|
| `async-openai` | 0.43.0 | 2026-11-01 | 10.2M (3.2M) | The one community crate OpenAI lists [17] |
| `openai-api-rs` | 10.0.2 | 2026-10-03 | 0.8M (0.13M) | Unofficial |
| `openai` | 1.1.1 | 2025-06-02 | 0.17M | Unofficial, stale |

Source: crates.io API, 2026-11-02 [18].

`async-openai` 0.43.0 is maintained and feature-gated (`responses`), but it
depends on `reqwest ^0.13` [18] while `core` uses `reqwest 0.12`
(`crates/core/Cargo.toml`), so it would put a second HTTP stack in the build.
It also brings the whole Responses type tree, most of which Advice never reads,
against the type rule in AGENTS.md.

Advice needs one `POST` and one `GET` with a bearer header. `core` already has
`reqwest` (rustls, json), `serde`, `serde_json` and `tokio`, and the existing
clients set their own timeouts (`live/poll.rs`, `delivery/table_channel.rs`).
Private serde structs for the request and the few response fields read
(`status`, the output text or refusal, `usage`, `error.code`) are small. The
base URL as a parameter lets the tests fake OpenAI with the local HTTP server
harness they already use, as the test rule requires.

Recommendation: plain HTTP.

## Open questions

- **Latency.** No page gives time-to-first-token or tokens per second for
  either model [2][3][9]. A probe with a real key should time `gpt-6-luna` and
  `gpt-6.1-terra` at the realistic prompt, `none` vs `low`, Standard vs Fast,
  before the failure-behaviour ticket picks a deadline.
- **Free tier.** Whether a key that never bought credit can call `gpt-6-luna`,
  and at what limits, is not published [2][12]. Likely the setup text should
  say a $5 credit purchase is needed.
- **Schema latency.** How much the first-request schema cost is, and whether
  it is per schema per organization or shorter-lived, is not stated [8]. With a
  fixed schema it matters once.

## Sources

All read 2026-11-02. `platform.openai.com/docs/*` now redirects (301) to
`developers.openai.com/api/docs/*`.

1. Models: https://developers.openai.com/api/docs/models
2. GPT-6 Luna: https://developers.openai.com/api/docs/models/gpt-6-luna
3. GPT-6.1 Terra: https://developers.openai.com/api/docs/models/gpt-6.1-terra
4. Pricing: https://developers.openai.com/api/docs/pricing
5. Migrate to Responses: https://developers.openai.com/api/docs/guides/migrate-to-responses
6. Your data: https://developers.openai.com/api/docs/guides/your-data
7. Reasoning: https://developers.openai.com/api/docs/guides/reasoning
8. Structured outputs: https://developers.openai.com/api/docs/guides/structured-outputs
9. Latency optimization: https://developers.openai.com/api/docs/guides/latency-optimization
10. Fast mode: https://developers.openai.com/api/docs/guides/fast-mode
11. Prompt caching: https://developers.openai.com/api/docs/guides/prompt-caching
12. Rate limits: https://developers.openai.com/api/docs/guides/rate-limits
13. Error codes: https://developers.openai.com/api/docs/guides/error-codes
14. Retrieve model: https://developers.openai.com/api/reference/resources/models/methods/retrieve
15. List models: https://developers.openai.com/api/reference/resources/models/methods/list
16. Permissions: https://developers.openai.com/api/docs/guides/rbac
17. Libraries: https://developers.openai.com/api/docs/libraries
18. crates.io API: https://crates.io/api/v1/crates/async-openai, `/openai-api-rs`, `/openai`, and `/async-openai/0.43.0/dependencies`
