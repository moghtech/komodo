# Pre-fix baseline — mechanisms reproduced (fresh runs, gate passed)

- **Date:** 2026-10-02 (batch runs 00:10:04–00:16:16, host local time -03:00;
  `exchange-error` re-run at 00:30:25 — provenance note in its section)
- **Suite commit / HEAD:** `fa53da3c0` — `compose/oidc-dev/verify.mjs` exactly as
  committed there (no local edits); branch `komodo-oidc-login-jwt-redeem-race-fix-1665`
  (pre-rewrite hash; full history on the fork's
  `komodo-oidc-login-jwt-redeem-race-fix-1665-archive` branch — same note applies
  to every commit hash in this document, see the post-fix provenance)
- **UI under test:** **stock v2.3.3** — last commit touching `ui/` is the `v2.3.3`
  tag commit itself (`780ac68b9`); the branch carries none of the fix's code yet
  (fix implementation not started). Core image = digest-pinned `ghcr.io/moghtech/komodo-core@sha256:bca73d0e…`
  (tag `2.3.3`).
- **Harness origin:** podman shim (`DOCKER_HOST=unix:///run/user/1000/podman/podman.sock`
  + standalone `docker-compose`); all browser-facing origins carry `:8443`
  (rootless podman cannot publish privileged ports — README Deviations 1). Mongo 8,
  core (branch UI build), oidc-mock (`oidc-provider@9.12.2`), Caddy 2.11, delay sidecar.
- **Method:** all 7 scenarios re-run **fresh** for this baseline (the verify suite's
  cached `out/*.ndjson` moved aside before the first run — none of the excerpts below is a
  cached read). Individual runs, ≥16 s apart (per-IP auth limiter 5 attempts/15 s;
  `all` mode's spacing, applied manually). Every scenario printed
  `SCENARIO <name> PASS` with exit 0; FIX-DEPENDENT checks report `SKIP` pre-fix by
  design (never PASSed without `--post-fix`).
- **Evidence files:** `out/<scenario>.ndjson` (gitignored). Excerpts below quote the
  evidentiary rows with two uniform elisions, both cut to `…` — the `out/` files carry
  the full values: (1) oversized jwt strings are truncated mid-value; (2) fields with
  no evidentiary weight in that excerpt (`host`, uncited `ts`/`ts_ms`, `duration_s`)
  are dropped from the middle of a row. Never elided: `url`, `status`, `auth`, and any
  timestamp the surrounding text cites.

**Gate verdict: ≥1 mechanism reproduced (M1, M2, M5) → PROCEED to the fix implementation.**

> **Post-fix results:** the second half of this document (same harness,
> `--post-fix` enforced, branch UI) is the "Post-fix run" section at
> the bottom — **all seven scenarios green**. `m1-seeded` reached green only via
> the `641f966da` pre-arm fix, which the first post-fix run exposed
> (FAIL → root cause → fix → PASS; resolution below), and the upstream issue
> drafts close the section.

| Scenario | Result | Mechanism signal |
|---|---|---|
| `success` | PASS (7/7) | environment row — no mechanism, regression guard |
| `latency` | PASS (8/8) | environment row with the knob — round-trip 2027 ms ≥ 2000 ms |
| `m1-seeded` | PASS (4/4) | **M1 REPRODUCED** — stale-token reads fire during pending |
| `m2-forced` | PASS (4/4 +1 SKIP) | **M2 REPRODUCED** — silent `add_and_change` drop |
| `exchange-error` | PASS (3/3 +2 SKIP) | **M5 REPRODUCED** — 401 → eternal spinner |
| `hung` | PASS (3/3 +3 SKIP) | slow success, NOT an eternal spinner (see the `hung` section below) |
| `isolation` | PASS (3/3) | regression guard — gate already isolated pre-fix |

---

## success (environment row)

Stock happy path is healthy: the exchange 200 lands, the very next app request is
authenticated, the ws connects, and the dashboard renders. The window-bound guards
observe **zero** unauth/401/403/auth-5xx rows — nothing else in the login
cascade is burning the limiter.

```
{"kind":"req","ts":1790910608.6233966,…,"method":"POST","url":"/auth/login/ExchangeForJwt","auth":false,"status":200,"duration_s":0.015468786}
{"kind":"req","ts":1790910608.7805629,…,"method":"GET","url":"/user","auth":true,"status":200,"duration_s":0.001466645}   ← 157 ms after the 200
{"kind":"req","ts":1790910616.7822032,…,"method":"GET","url":"/ws/update","auth":false,"status":101,"duration_s":7.367679162}
checks: window total 0 · sliding-15s max 0 · dashboard rendered at /
```

## latency (environment row + knob)

Identical shape with `DELAY_AUTH_MS=2000`; the point is that the delay sidecar is
really in the path and only on `/auth/login/*`:

```
{"kind":"req","ts":1790910652.511251,…,"method":"POST","url":"/auth/login/ExchangeForJwt","auth":false,"status":200,"duration_s":2.026986392}
check: observed exchange round-trip >= DELAY_AUTH_MS (2000ms; observed 2027ms) PASS
```

## m1-seeded — **M1 REPRODUCED**

Seed: a structurally-valid, bad-signature jwt (`current:"stale-user"`, tail
`hwIjoxfQ.stale-signature`) in `mogh-auth-tokens-v1` before the drive;
`DELAY_AUTH_MS=1500` widens the redeem window. The stale-token requests fire from
the always-mounted provider subtree **before** the exchange settles:

```
{"kind":"fetch","ts_ms":1790910697471,"path":"/user","auth_tail":"hwIjoxfQ.stale-signature"}
{"kind":"fetch","ts_ms":1790910697472,"path":"/read/GetCoreInfo","auth_tail":"hwIjoxfQ.stale-signature"}
{"kind":"req","ts":1790910697.4745677,"method":"GET","url":"/user","auth":true,"status":500,"duration_s":0.001200671}
{"kind":"req","ts":1790910697.4746535,"method":"POST","url":"/read/GetCoreInfo","auth":true,"status":500,"duration_s":0.000997077}
{"kind":"req","ts":1790910699.008,…,"method":"POST","url":"/auth/login/ExchangeForJwt","auth":true,"status":200}   ← window closes 1.53 s LATER
```

Wire rows prove auth-header **presence** + rejection + timing (Caddy redacts the
value); the fetch-shim rows prove the **value** — those two requests carried the
seeded stale token, not the fresh one. The same shim shows the stale token is also
attached to the `ExchangeForJwt` call itself (ts 1790910697477), corroborated on the
wire by that row's own `auth:true` — mogh_ui attaches
whichever jwt is current to *all* API calls, which is why the residual-count unit counts
auth-bearing 5xx, not just 401/403. After settlement the pointer moves off the seed:

```
{"kind":"tokens","ts_ms":1790910696769,"detail":"TOKENS 1790910696767 {\"current\":\"stale-user\",…}"}
{"kind":"tokens","ts_ms":1790910699010,"detail":"TOKENS 1790910699009 {\"current\":\"6abf067c65cacd7846f1e424\",\"tokens\":[{…stale…},{\"user_id\":\"6abf067c…\",\"jwt\":\"…\"}]}"}
```

**Rejection code is 500, not 401.** Core answers the bad-signature jwt with 500 on
`/user` and `/read/GetCoreInfo` on this build. The original "the server 401s
it" design assumption was already amended to "rejections (observed 500s)"; the
suite's predicate deliberately accepts 401/403/5xx and prints the observed statuses
so the discriminating signal stays *stale-token-authenticated request inside the
window*, not the exact code.

### M1 vs the read-deferral premise — CONFIRMED

The fix defers the always-mounted provider-subtree reads (`useUser` poll, `GetCoreInfo`,
connect effect) behind `enabled: gateOpen && hasJwt` for exactly this surface. The
observation matches the premise precisely: **stale-token reads fire during the
pending window** (`[drive start → exchange 200]`), from `WebsocketProvider`'s
mounted queries, with errors that would latch (`retry: false`). Nothing in the fresh
evidence contradicts the deferral premise; the deferral targets what actually happens.

## m2-forced — **M2 REPRODUCED**

Route interception rewrites the real 200's `body.jwt` to a structurally-valid
sub-less jwt. Upstream is 200; the client never stores it; the reload lands on
`/login` with **no signal**:

```
{"kind":"req","ts":1790910746.422467,…,"method":"POST","url":"/auth/login/ExchangeForJwt","auth":false,"status":200,"duration_s":0.026053597}
{"kind":"tokens","ts_ms":1790910739430,"detail":"TOKENS 1790910739429 null"}
{"kind":"tokens","ts_ms":1790910740042,"detail":"TOKENS 1790910740041 null"}        ← … every TOKENS row stays null
{"kind":"req","ts":1790910746.5997264,…,"method":"GET","url":"/assets/login-DoZQWzrW.js","auth":false,"status":200}   ← post-exchange reload loads the LOGIN chunk
{"kind":"req","ts":1790910746.64838,…,"method":"POST","url":"/auth/login/GetLoginOptions","auth":false,"status":200}
checks: sub-less jwt never lands in the token store PASS · reload lands on /login PASS
```

Every TOKENS row captured reads `null` and a grep for the sub-less jwt's payload
across the whole ndjson returns 0 hits — `add_and_change`'s early return dropped it
silently. The only user-visible consequence is the login page; the post-fix
"drop flag surfaced on the login page" check is the `SKIP`ped fix-dependent arm.

## exchange-error — **M5 REPRODUCED**

> Provenance: `out/` is a rolling cache — any later run of the suite overwrites a
> scenario's ndjson. The original batch run of this row (00:12–00:14) was overwritten
> by an out-of-band re-run, so this section was **re-run by this task** (same committed
> suite, same PASS verdict, stdout captured) and cites that run below. All other
> sections cite the batch runs whose files are still on disk.

Leg 1 completes a full login (dashboard). Replaying `/?redeem_ready=true` re-fires
the exchange against the ONE-SHOT pending state → 401 (this also burns one of the
5-per-15s limiter attempts; a UI loop that re-fires exhausts the budget and then
429s — the closest analogue to the reporter's "7 exchanges / 0 authed"):

```
{"kind":"req","ts":1790911828.2776556,…,"method":"POST","url":"/auth/login/ExchangeForJwt","auth":false,"status":200}   ← leg 1
{"kind":"req","ts":1790911833.434673,…,"method":"GET","url":"/?redeem_ready=true","auth":false,"status":200}             ← replay navigation
{"kind":"fetch","ts_ms":1790911833470,"path":"/auth/login/ExchangeForJwt","auth_tail":"k1pJsydNq3BaMmVjUKVrhvYg"}        ← still-stored jwt attached
{"kind":"req","ts":1790911833.4868107,…,"method":"POST","url":"/auth/login/ExchangeForJwt","auth":true,"status":401,"duration_s":0.016108626}
{"kind":"loader","ts_ms":1790911833544,"state":"on","detail":"LOADER 1790911833544 on"}                                  ← last loader row, ever
```

After the 401: **no navigation for the 8 s observation** (URL still
`/?redeem_ready=true`), the LoadingScreen's last observed state is `on` (eternal
spinner), and no second document load occurs. The 60 s residual window observed 0
unauth/401/403/auth-5xx rows and 0 failed ws handshakes — pre-fix this window is an
observation only (enforced under `--post-fix`); it confirms nothing *else* re-fires
the exchange while the spinner spins.

## hung — slow success, NOT an eternal spinner

`DELAY_AUTH_MS=15000` (past the fix's 12 s watchdog). Pre-fix there is **no watchdog**,
so the LoadingScreen simply holds for the entire delay and the late 200 then
converges normally — the scenario map's pre-fix "eternal spinner" cell for `hung` is
**refuted for finite delays**; the eternal arm is only reachable by an exchange that
never settles, which a finite delay sidecar cannot produce (that arm belongs to the
post-fix watchdog run). The README's row→scenario map already carries the
corrected pre-fix cell ("round-trip ≥ 15000 ms; LoadingScreen up with no OBSERVED
gap until the late 200 … slow-success reload documented") — BASELINE does not
restate it.

```
{"kind":"loader","ts_ms":1790910870627,"state":"on"}                                   ← redeem landing
{"kind":"req","ts":1790910885.644007,…,"method":"POST","url":"/auth/login/ExchangeForJwt","auth":false,"status":200,"duration_s":15.036977271}
{"kind":"tokens","ts_ms":1790910885647,"detail":"TOKENS 1790910885646 {\"current\":\"6abf067c…\",\"tokens\":[…]}"}   ← stored 3 ms after the 200
{"kind":"loader","ts_ms":1790910885832,"state":"on"}                                   ← still on, post-200, pre-reload (no gap)
{"kind":"loader","ts_ms":1790910886195,"state":"off"}                                  ← sanitize reload unmounts the spinner
checks: round-trip 15037 ms ≥ 15000 PASS · loader on with no OBSERVED gap (15017 ms; gap-off=false) PASS
pre-fix observation: after the late 200 the URL is / (slow-success reload)
```

Loader-on observed span: 886194 − 870626 = **15568 ms** — one continuous spinner
across the whole 15 s delay, ending only via the success reload. Caveat carried from
the suite: the shim polls at 100 ms, so sub-100 ms transitions could hide ("no
OBSERVED gap").

## isolation (regression guard)

A failing **non-redeem** execute (`POST /execute/StartDeployment` → 500) fired with
the stored jwt does not flip the gate, reload, or raise the LoadingScreen — stock
v2.3.3 already isolates mutations from the redeem path. Pre-fix PASS is expected
(README Deviations 6); the row exists to catch an unfiltered redeem-gate MutationCache
subscription in the fix implementation.

```
check: probe failing execute returned non-auth 4xx/5xx (observed 500) PASS
check: no full-page reload to /login within 2s (URL /; marker alive) PASS
check: no LoadingScreen flip within 2s (flip observed: false) PASS
```

---

## Mechanism attribution

| # | Mechanism | Verdict | Evidence |
|---|---|---|---|
| M1 | Stale-token query fire | **REPRODUCED** | `m1-seeded`: stale-value `/user` + `/read/GetCoreInfo` (fetch-shim tail `…stale-signature`) hit core during pending, rejected 500, 1.53 s before the exchange 200 |
| M2 | Silent `add_and_change` drop | **REPRODUCED** | `m2-forced`: upstream 200, zero TOKENS transitions (all `null`), sub-less jwt absent from the store, reload → login chunk + `GetLoginOptions`, no signal |
| M3 | Forward-auth bounce → reload loop | **NOT REPRODUCED** (mock limitation — see below) | `success`/`exchange-error`: the post-exchange and post-replay reloads return 200 straight through the gate; zero `/verify` 503/401 bounces in any ndjson |
| M4 | Latched errors never re-run | **NOT INDEPENDENTLY REPRODUCED** | No dedicated scenario row by design; per the mechanism analysis it is covered by the success-path remount, and `success`'s post-reload authed reads + dashboard + ws assert the recovered state (no residual latched state observed) |
| M5 | Exchange failure → eternal spinner | **REPRODUCED** | `exchange-error`: replayed exchange 401 → no navigation, last loader state `on` indefinitely |
| M6 | Core auto-redirect loop | **OUT BY CONSTRUCTION** | `core-config.toml` pins `oidc_auto_redirect = false`, so the harness cannot produce it, and no loop signature appeared |

### M3 non-reproduction — what it does and does not mean

The mock's portal session **survives token redemption**, so the forward-auth gate
never bounces the post-exchange sanitize reload: every reload of
`/?redeem_ready=true` returns 200 and no scenario shows a `/verify` 401/503 storm or
a portal↔komodo loop. This is a **mock limitation, not an exoneration**: the
reporter's production gateway (Authelia-class) may consume/invalidate its session at
the redemption instant, which is exactly the M3 trigger this mock cannot express.
M3 therefore stays **live for the upstream report** (the report records the
contingency: no komodo-side fix under the pinned dependency), and any loop observed
in a later run must be re-checked against core logs (`invalid peer certificate` /
discovery 500s are the known non-M3 confounders — README's debugging note) before
being attributed.

## Gate decision

**PROCEED.** Three of the six candidate mechanisms reproduce with wire-level
evidence (M1, M2, M5), matching the design premise that the fix must cover all
three regardless of which reproduces first (see the attribution table above). The environment rows (`success`,
`latency`, `isolation`) are green pre-fix, so the post-fix runs have a clean
regression baseline, and `hung`'s pre-fix reality is documented for the hung row's
comparison.

---

*Metadata: produced by the fresh pre-fix runs (2026-10-02 00:10–00:16 -03:00;
`exchange-error` re-run 00:30 -03:00); harness
per README "Run" (podman shim, `compose/oidc-dev.compose.yaml`, digest-pinned core
`sha256:bca73d0e…`, `oidc-provider@9.12.2`); suite at HEAD `fa53da3c0` (pre-rewrite
hash; full history on the fork's `komodo-oidc-login-jwt-redeem-race-fix-1665-archive`
branch); raw ndjson evidence in gitignored `compose/oidc-dev/out/`.*

---

# Post-fix run — `--post-fix` arms enforced

Completes the second half of this document: same harness, same drive, same suite
— core rebuilt to serve the branch fix, every run carrying `--post-fix` so the
fix-dependent assertions were ENFORCED (never SKIPped) and the pre-fix mechanism
assertions SKIPped by design (`reportChecks`).

> **Provenance note:** every commit hash cited in this document (`fa53da3c0`,
> `9d9a38a88`, `49802408e`, `641f966da`, …) is a pre-rewrite hash, unreachable
> from the PR branch's current history; full history lives on the fork's
> `komodo-oidc-login-jwt-redeem-race-fix-1665-archive` branch.

- **Date / builds:** batch 0 — 02:14:55–02:25:29 -03:00 against the
  **pre-pre-arm build** (`5190032c1dbe`, HEAD `9d9a38a88`); batch 1 (final) —
  02:51:17–02:54:55 -03:00 against the **final build** (`28f89a5be8af`, HEAD
  `641f966da`); batch 2 (canonical) — 08:58–09:04 -03:00 against the SAME final
  build (image id re-verified unchanged: `28f89a5be8af`). Individual runs, ≥16 s
  apart — the per-IP limiter discipline. Batch 0 first ran every scenario and
  EXPOSED a real fix defect (m1-seeded FAIL, root-caused below); `641f966da`
  (module-scope pre-arm, reviewed) closed it, and batch 1 re-baselined four rows
  on the final image (`success`, `m1-seeded`, `hung`, `isolation`). The remaining
  three rows originally stood on batch 0 under a transfer argument that did not
  survive scrutiny — batch 0's own `exchange-error` ndjson shows the pre-arm
  gap firing in that scenario (−16 ms rows, detailed below the verdict table) —
  so batch 2 re-ran them on the final build. Every post-fix row in the table is
  now a final-build run; batch 0's copies are historical only.
- **Suite:** batch 0 ran `verify.mjs` byte-identical to `fa53da3c0`; batch 1
  runs it with the one-predicate scoping fix from `641f966da` (documented in
  the m1 resolution — the m1 value arm's shim filter self-matched the
  exchange's own stale-jwt dispatch row). All runs carry `--post-fix`
  (fix-dependent checks ENFORCED; pre-fix mechanism assertions SKIPped by
  design).
- **UI under test (batch 1, re-verified for batch 2):** the branch fix with pre-arm
  (`49802408e…641f966da`: gate lifecycle incl. the module-scope pre-arm,
  settlement listener, evidence flag, late-success redirect). Core image `localhost/komodo-oidc-dev-core` id `28f89a5be8af`,
  built from HEAD `641f966da`; the running core container was verified to run
  that image (`docker inspect` image sha match) before the batch. Batch 0's
  image (`5190032c1dbe`, HEAD `9d9a38a88`) had its served
  `index-CfUQi4Ph.js` hash matched against the host `yarn build` output.
- **Host build acceptance:** `cd ui && yarn build` clean (exit 0, verified on
  both builds).
- **Evidence:** `out/<scenario>.ndjson` files refreshed by each batch
  (gitignored — batch 1 is the surviving file for its four rows, batch 2 for
  its three); excerpts
  below follow the same elision discipline as the pre-fix sections.

## Verdicts (pre-fix → post-fix)

Every post-fix row below ran on the FINAL build. Batch 1 covered four rows;
batch 2 re-ran the remaining three (`latency`, `m2-forced`,
`exchange-error`) after the transfer argument that had let them stand on batch 0
was refuted — see "The retired † argument" below. Batch 0's copies of those
rows are historical only. **Round-8 batch 3 re-ran ALL SEVEN at the PR head
(canonical; see the round-8 batch section below) after F-001 showed batch 2
predated three detector commits. Round-9 batch 4 added `m2-seeded` (the
classifier's previously blind population) and re-ran ALL EIGHT at the PR
head (canonical; see the round-9 batch section). Round-10 batch 5
re-certified 8/8 at the PR head after the marker scrub and the login-page
`backto` input sanitize (see the round-10 batch section).**

| Scenario | Pre-fix | Post-fix (final build) | Fix-dependent arms (`--post-fix`) |
|---|---|---|---|
| `success` | PASS 7/7 | **PASS 7/7** (batch 1) | n/a — regression guard holds: window total 0, sliding 0, ws 101 + on_login, dashboard `/` |
| `latency` | PASS 8/8 | **PASS 8/8** (batch 2) | n/a — regression guard holds: round-trip 2058 ms ≥ 2000 ms |
| `m1-seeded` | PASS 4/4 (M1 REPRODUCED) | **PASS 4/4** (batch 1; was **FAIL 2/4** on batch 0) | zero-stale-reads arm (read deferral): **observed 0** stale reads; value-absence arm PASS (scoped); fresh-pointer PASS |
| `m2-forced` | PASS 4/4 +1 SKIP (M2 REPRODUCED) | **PASS 5/5** (batch 2) | drop flag surfaced in-document on the login page — M2 no longer silent |
| `exchange-error` | PASS 3/3 +2 SKIP (M5 REPRODUCED) | **PASS 3/3** (batch 2; 2 pre-fix arms SKIP) | converged failure in-document (exactly 1 document load, final `/login`) + zero-residual 60 s (0 bad rows, 0 failed ws) |
| `hung` | PASS 3/3 +3 SKIP (slow success) | **PASS 5/5** (batch 1; 1 pre-fix arm SKIP) | watchdog dropped the gate 12.1 s after landing — 3.0 s BEFORE the late 200; zero-residual 60 s (0/0); the late-success redirect recovered the login → landed `/` |
| `isolation` | PASS 3/3 | **PASS 3/3** (batch 1) | n/a — the gate's filtered subscription introduced no gate flip (regression guard holds) |

**7/7 green — the "full suite green post-fix" acceptance criterion is met.**
Re-certified at the PR head by round-8 batch 3 (`a861602a4`): same verdict,
SHA-bound this time. Round-9 batch 4 (`1f0f2673b0bb`) grew the suite to
eight scenarios (the classifier's blind population added) and re-certified
8/8 at the PR head.

### The retired † argument — what was wrong, and what replaced it

The previous revision marked `latency` / `m2-forced` / `exchange-error` †
("ran on the pre-pre-arm build; kept because their asserted mechanisms do not
touch the pre-arm window"). That justification was **false for
`exchange-error`**, and batch 0's own ndjson refutes it: the replay document
dispatched auth-bearing `GET /user` + `POST /read/GetCoreInfo` BEFORE the redeem
dispatch — fetch-shim `ts_ms 1790918528200` and `…528200` vs the exchange
dispatch at `…528202` (−2 ms); wire completions `1790918528.2028` /
`1790918528.202812` vs the 401 at `1790918528.2185` (−16 ms) — both rows
`Referer`-pinned by Caddy to `/?redeem_ready=true`, i.e. sent by the redeem
document itself. That IS the parent-first gap the pre-arm closes, firing in
exactly the scenario the old text claimed was untouched. (Recorded in full as
PRE-ARM PROOF in the m1 section below.)

The two-part transfer argument, stated correctly — and mooted by batch 2:
1. **Pre-arm window:** the module-scope pre-arm defers the redeem document's
   own provider reads until the gate closes. Batch 0's `exchange-error` rows
   prove the gap was real in a second scenario (not just `m1-seeded`); batch 2
   proves the fix closes it there too — zero fetch rows of any kind between the
   replay document load and the exchange dispatch.
2. **Post-settlement (SUPERSEDED by round-8 C-003):** this part originally
   said reads stay off post-settlement because the settlement hygiene
   (`LOGIN_TOKENS.remove_all`) empties the store. Round 8 removed that wipe:
   it ran on EVERY failure path, so a failed redeem silently logged every
   tab of the browser out of a perfectly valid pre-existing session (stale
   `?redeem_ready=true` bookmark, replayed one-shot, watchdog on a
   black-holed proxy) — and mogh's `add_and_change` is all-or-nothing, so
   there was no partial state for it to clean. The post-fix contract:
   post-settlement reads resume with whatever token the store holds. A
   surviving VALID session yields authenticated 200s (which the
   zero-residual predicate never counted); the batch-3 preservation oracle
   asserts the survival itself (≥1 authed `/user` 200 strictly after the
   settlement row — fail-closed, since an emptied store can produce no
   authed request at all).

Since part 1 was a live defect inside a † row's own scenario, no transfer was
sound; the rows were re-run (batch 2), and batch 0's three rows are kept as
history only.

## Round-8 batch — 7/7 re-certified at the PR head (and what the first attempt caught)

Round-8 review (F-001) held that batch 2 — this document's canonical 7/7 —
certified `641f966da` while three later commits (`e9efeac04`, `ccc94c0cc`,
`308eddd6d`) reworked the exact silent-drop detector the m2 row validates,
and nothing forced the next run to bind to the tree it exercised. Two changes
close that permanently:

- **SHA-bound evidence (verify.mjs):** every run stamps `git rev-parse HEAD`
  into its scenario banner and every ndjson meta row (all batch-3 rows carry
  `head=a861602a4279, dirty=0, postFix=true`), and the runner REFUSES to
  execute with uncommitted changes under `ui/src` or `compose/oidc-dev` —
  the gate refused one launch during this very round (the doc edits for this
  section were uncommitted) and died at module top before touching the
  stack, which is the designed behavior. Dev escape:
  `VERIFY_ALLOW_DIRTY=1` proceeds but stamps the dirt count — evidence can
  be recorded dirty, never mis-recorded clean.
- **Run protocol:** commit → build → run → record. The stamped SHA is the
  join key between a PASS line and the code that earned it.

### Batch 3a (`e6192cf37`, image `8835351d67ac`, ~16:50 -03:00) — 6/7, and the failure was real

This was the first post-fix execution of the drift-guard detector at all
(batches 0–2 never ran it — that is F-001's point), and it FAILED
m2-forced's fix-dependent surfacing arm: `no komodo-redeem flag and no
notification text`. The ndjson root-causes a consume race BETWEEN documents,
not a detector defect: the settlement writes the drop flag in the same
dispatch task as the exchange 200 (exchange at `…726.843`); the OLD document
— session-less, settled — bounces RequireAuth to `/login` and mounts Login
(login chunk fetched at `…726.857`); mogh's sanitize reload lands at
`…726.892`; and the doomed document's clear-on-read consumed the flag
milliseconds before the unload. Batch 2 had passed only because the unload
happened to win that passive-effect race. Fix: `dfa654392` — the drop branch
no longer consumes (TTL expiry is the cleanup; freshness gates the toast;
the ok branch keeps consume-before-navigate, because THAT redirect loops).

### Batch 3b (canonical — HEAD `a861602a4`, image `778aee5742c6`, 17:08–17:14 -03:00)

```
PASS stale-token requests during the redeem window — expect ZERO (fix defers provider reads) (wire: auth+rejected in [drive start -> exchange 200]; observed 0, statuses -)
PASS stale token VALUE on those paths — expect ABSENT (fetch shim tail …hwIjoxfQ.stale-signature)
PASS post-fix: drop flag surfaced in-document on the login page
PASS post-fix: in-document convergence, session preserved (final /; document loads for the replay: 1)
PASS zero-residual 60s window (unauth-or-401/403-or-auth-5xx app rows: 0; failed ws handshakes: 0)
PASS post-fix: leg-1 session preserved through the failed settlement (authenticated /user 200 after ts=1790971835.641; observed 5)
```

`SCENARIO … PASS` ×7 (`success` 7/7, `latency` 8/8 with round-trip 2029 ms,
`m1-seeded` 4/4, `m2-forced` 5/5, `exchange-error` 3/3 + 2 pre-fix SKIPs,
`hung` 5/5 + 1 pre-fix SKIP, `isolation` 3/3), `ALL SCENARIOS PASS`, exit 0.
Round-8 contract rows worth calling out:

- **exchange-error session preservation (C-003):** the replayed 401 settles
  store-neutral; the observation window held 0 unauth/401/403/5xx rows, 0
  failed ws handshakes, and **5 authenticated `/user` 200s** — the leg-1
  session survives and the app renders at `/`. The preservation arm is
  fail-closed against reintroducing the wipe: an emptied store can produce
  no authenticated request at all.
- **m2-forced surfacing (the batch-3a lesson):** the drop flag is present in
  the surviving document and the toast renders — race-free by construction
  now, because nothing consumes the flag before the TTL.
- **m1 check names (F-003):** names state the mode-true expectation
  ("expect ZERO (fix defers provider reads)" under `--post-fix`), so pasted
  stdout no longer asserts its own inverse.

**7/7 green — now certifying the code being merged, not two commits of
history.**

## Round-9 batch — 8/8 at the PR head; the classifier's blind population gets its own scenario

Round-9 review found three majors, all in the classifier/evidence layer:

- **C-003 (production):** the round-7 drift guard's second disjunct
  (`jwtAbsent && !!jwt()`) misread the most common drop population as
  upstream key-drift: in a redeem document opened with a VALID pre-existing
  session, mogh's IIFE closure holds the OLD token (truthy), so a silently
  dropped exchange stayed `phase: "ok"` — no toast, and the one console
  error named the wrong hypothesis. "M2 closed at the komodo layer" was
  session-less-only. Fix: drift discriminates by EQUALITY against the
  exchanged jwt — a successful `add_and_change` sets the closure to the
  exchanged jwt in-process, a dropped one leaves it untouched, so
  `closure === exchanged` while the pinned key lacks that jwt is the
  precise key-rename signature, and the prior-session population classifies
  as the drop it is. No snapshot machinery needed.
- **C-001 (harness):** the m2 surfacing arm accepted any truthy flag phase —
  an unconsumed "ok" on /login is precisely the signature of a classifier
  that failed to verdict drop (the ok branch consumes its flag BEFORE
  navigating), and it printed PASS. Now asserts `phase === "drop"`.
- **C-002 (harness):** the SHA-gate pathspec covered `ui/src` +
  `compose/oidc-dev`, but the run exercises more merge surface: `ui` whole
  (the bundle's build inputs), `client/core/ts` (the yarn-linked client the
  same builder stage compiles), the stack topology
  (`compose/oidc-dev.compose.yaml` sits OUTSIDE `compose/oidc-dev/`), and
  `.dockerignore`. Widened; the acknowledged residual (the gitignored
  `.env` digest pin) is documented in place.

Minors: `composeEnabled`'s function branch falsy-coerces like react-query's
`resolveEnabled` (C-004, latent); `unauthOrFail` counts 429 — the limiter's
own rejection code — as a bad row (C-005); `exchange-error` and `hung` flush
their ws row at close and assert the FLUSH RESULT, so `wsFails: 0` is no
longer vacuous (C-006).

The new `m2-seeded` scenario drives the C-003 population the suite had
structurally never seen: the same route-intercepted sub-less exchange, with
a session pre-seeded into the store. Its added arms: the drop verdicts and
surfaces WITH a pre-existing session, and the seeded session is preserved
through the drop (store neutrality).

### Batch 4a (`78739f1fc0f8`, image `925a844ef0a0`, ~17:57 -03:00) — 7/8, and the failure was the check, not the fix

The new ws-alive check failed deterministically on a healthy
`exchange-error` run. The instrumented probe (one `VERIFY_ALLOW_DIRTY`
diagnostic run, not evidence) showed the entries holding exactly one ws row
— the preserved session's — with ts 44 ms BEFORE the settlement row: Caddy's
ts on a ws row is the UPGRADE (request) time, and leg-2's provider ws
upgrades during pending (the ws does not wait for the gated reads). The
"after settlement" timestamp predicate mis-failed a healthy run; `hung`'s
identical check passed only because its recovery ordering happened to
upgrade after the late 200 — luck, not correctness. Fix (`1f0f2673b0bb`):
`flushWsRow` returns whether the 101 row was observed and both scenarios
assert THAT result — the invariant C-006 needs is "a ws connection existed
and closed 101", which the flush itself proves.

### Batch 4b (canonical — HEAD `1f0f2673b0bb`, image `925a844ef0a0` (built from the `c9873f808` tree; `ui` is unchanged by the two later harness-only commits), ~18:29–18:43 -03:00)

```
PASS post-fix: drop flag surfaced in-document on the login page (phase drop)
PASS post-fix: drop surfaced WITH a pre-existing session (round-9 C-003 population)
PASS pre-existing (seeded) session preserved through the silent drop (store neutrality)
    ↳ observation (this phase): 0 unauth-or-401/403-or-auth-5xx rows, 0 failed ws handshakes, 5 authenticated /user 200 (leg-1 session preserved) in the 60s window
```

`SCENARIO … PASS` ×8 (`success` 7/7, `latency` 8/8 with round-trip 2029 ms,
`m1-seeded` 4/4, `m2-forced` 5/5, `m2-seeded` 7/7, `exchange-error` 5/5 +
2 pre-fix SKIPs, `hung` 6/6 + 1 pre-fix SKIP, `isolation` 3/3),
`ALL SCENARIOS PASS`, exit 0; all eight ndjson meta rows stamped
`head=1f0f2673b0bb, dirty=0`.

**8/8 green — the suite now exercises the population its own classifier
used to misclassify.**

## Round-10 batch — 8/8 at the PR head; the shipped source stops speaking review dialect

Round-10 review found two majors about readers rather than runtime, three
minors:

- **C-001:** review-round markers (`round-8 C-003`, `round-9 C-004`,
  `batch-3 lesson`, `upstream draft (b)`) had accreted into shipped
  `ui/src` comments — ids that resolve only through this document, and
  collide across rounds (round-8 C-003 ≠ round-9 C-003; a grep lands 50/50
  on the wrong rationale). An earlier scrub commit had replaced spec
  anchors but missed the hyphenated spelling class — the exact miss its own
  commit message warned about. Swept to zero (`grep -rnE
  'round-[0-9]|batch-[0-9]|C-00[0-9]|F-00[0-9]|draft \([a-z]\)' ui/src`);
  the WHY content stays, the process pointers are gone.
- **C-002 (live on the shipped app):** `safeLocalPath` guarded this PR's
  own redirect while the SAME parameter flowed unvalidated through
  mogh_ui's `maybeNavigate` after a local/passkey/totp login — the
  documented draft-(f) flaw, live on the page komodo owns. Fixed at the
  input: `Login`'s mount effect rewrites a hostile `backto` out of the URL
  (`history.replaceState`, no navigation) before anything reads it, so
  every consumer — mogh_ui's included — navigates only on a value that
  survived the guard.
- Minors: oidc-mock's baked issuer default now carries `:8443`, matching
  the harness's own byte-identical-issuer rule (C-003); hung's self-derived
  ">= 20s post-callback" check deleted — the oracle was the scenario's own
  unconditional sleep, and the window it gestured at is measured by the
  zero-residual check (C-004; hung is 5/5 + 1 pre-fix SKIP); the runner's
  evidence write is guarded and ordered before the browser/knob cleanup, so
  a write throw can no longer leak Chromium or leave `DELAY_AUTH_MS` set
  (C-005).

### Batch 5 (canonical — HEAD `9bf2f4a26`, image `c30be6e0dc43`, ~21:20–21:27 -03:00)

`SCENARIO … PASS` ×8 (`success` 7/7, `latency` 8/8, `m1-seeded` 4/4,
`m2-forced` 5/5, `m2-seeded` 7/7, `exchange-error` 5/5 + 2 pre-fix SKIPs,
`hung` 5/5 + 1 pre-fix SKIP, `isolation` 3/3), `ALL SCENARIOS PASS`, exit
0; all eight ndjson meta rows stamped `head=9bf2f4a26fd7, dirty=0`; 44
checks green. Bundle identity verified BYTE-LEVEL (the container's
image-ID field proved an unreliable witness under podman — it reported an
ID matching neither the tag nor the prior build): the served
`login-*.js` md5 equals the host `yarn build` output at this head — the
same served-hash technique batch 0 used.

**8/8 green.**

## Round-11 batch — 8/8; zero CRITICAL, zero MAJOR, and one roast-vs-roast disagreement settled by the pinned source

Round-11 review found **no production defects**. Four minors:

- **C-001 REFUTED — and the comment strengthened anyway.** The roast claimed
  the settlement listener is delivered via `systemSetTimeoutZero` (a later
  macrotask), contradicting an earlier round's receipt — the delivery
  mechanism decides whether the silent-drop verdict is a react-query
  guarantee or a browser-scheduling coincidence, so both claims could not
  stand. Re-verified against the pinned query-core 5.102.4:
  `mutationCache.notify` invokes `this.listeners.forEach` DIRECTLY inside
  `notifyManager.batch`, and `batch` runs its callback synchronously
  (`transactions++` → `callback()` → `transactions--`). The
  `systemSetTimeoutZero` scheduler delivers only the notifyManager's
  observer-notification `queue` — filled solely by `schedule()`/`batchCalls`,
  which cache listeners never touch; `flush()` with an empty queue is a
  no-op. The comment now names where the scheduler applies and where it does
  not, so the next reader who greps `notifyManager.cjs` and sees a scheduler
  does not conclude the cache listeners ride it.
- **F-002:** the harness pinned everything except its own driver — the host
  Playwright install was a caret range in a gitignored manifest. Now
  committed and pinned exact (1.63.0 — the resolved version every canonical
  batch ran on), lockfile in the repo, README install step is
  `npm ci && npx playwright install chromium`.
- **F-003:** `safeLocalPath` computed once in the Login mount effect and its
  ANSWER written — one invocation, no approval/writer divergence on the
  security seam.
- **F-001:** the PR body's "7-scenario" sentence (a stale count) corrected to
  8 — repo docs were already consistent.

### Batch 6 (canonical — HEAD `7ee624c16`, ~09:47–09:59 -03:00)

`SCENARIO … PASS` ×8 (`success` 7/7, `latency` 8/8, `m1-seeded` 4/4,
`m2-forced` 5/5, `m2-seeded` 7/7, `exchange-error` 5/5 + 2 pre-fix SKIPs,
`hung` 5/5 + 1 pre-fix SKIP, `isolation` 3/3), `ALL SCENARIOS PASS`, exit 0;
all eight ndjson meta rows stamped `head=7ee624c168f7, dirty=0`; 44 checks
green. Bundle identity verified byte-level: the container's served
`login-*.js` md5 equals the host `yarn build` output at this head (the
container also retains the previous build's stale chunk from COPY layering —
inert; the served index references only the current one).

**8/8 green.**

## m2-forced — the silent drop now surfaces (M2 closed at the komodo layer)

Pre-fix: upstream 200 with a sub-less jwt → `add_and_change` early-returns on
falsy `sub` (`mogh_auth_client/dist/tokens.js`: `if (!user_id) return;`), zero
TOKENS transitions, reload lands on `/login` with **no signal**.

Post-fix run (batch 2, final build — canonical; batch 0 had already passed the
same arms on the pre-pre-arm build, historical): the same drive (route
interception rewrites `body.jwt` to the sub-less token; upstream still 200),
exchange at `ts=1790942436.108`:

```
PASS token-store shim evidenced (>=1 TOKENS row captured)
PASS upstream exchange 200 observed (the drop is client-side; ts=1790942436.108)
PASS exchanged (sub-less) jwt never lands in the token store (silent drop)
PASS silent drop path: reload lands on /login (observed /login)
PASS post-fix: drop flag surfaced in-document on the login page
```

The drop is detected by the settlement listener (`initRedeemGate`: success
dispatch carried `jwt`, storage does not contain it → `writeFlag("drop")`) and
surfaced by `login.tsx` as the red toast "Login succeeded but the session could
not be stored". The probe finds the signal in-document at T+8 s because the
toast persists (`autoClose: false` — HEAD commit `9d9a38a88`; the Mantine 4 s
default raced exactly this probe). The library-level drop itself is upstream's
— draft issue (a) below.

## exchange-error — converged failure in-document (M5 closed at the komodo layer)

Pre-fix: replayed `/?redeem_ready=true` → 401 → no navigation for 8 s, last
loader state `on` indefinitely (eternal spinner), URL keeping `redeem_ready=true`.

Post-fix run (batch 2, final build — canonical): the two pre-fix arms SKIPped
as designed, both fix-dependent arms PASSED (replay 401 at
`ts=1790942487.047`):

```
PASS replayed exchange 401 observed (one-shot consumed session; ts=1790942487.047)
SKIP (pre-fix mechanism assertion; not applicable with --post-fix) pre-fix: no navigation for 8s after the 401 (URL still redeem_ready; observed /login)
SKIP (pre-fix mechanism assertion; not applicable with --post-fix) pre-fix: eternal LoadingScreen (still up at end of observation; last state: off)
PASS post-fix: in-document convergence to /login (final /login; document loads for the replay: 1)
PASS zero-residual 60s window (unauth-or-401/403-or-auth-5xx app rows: 0; failed ws handshakes: 0)
```

Batch 0's copy of this row (pre-pre-arm build — historical, and instructive):
the SAME two fix-dependent arms PASSED, but the run's own ndjson also carried
the −16 ms rows documented above and as PRE-ARM PROOF below — the reason this
row could not stand on batch 0. Batch 2's ndjson shows the pre-arm engaged:
between the replay document load (`1790942486.976`) and the exchange dispatch,
the fetch shim logged **zero** rows — the exchange is that document's first API
call — where batch 0 had the two auth-bearing provider reads at −2 ms (shim) /
−16 ms (wire).

The settled-failed path converged the SAME document to `/login` — the
replay produced **exactly one** document load (no sanitize reload, no
retry-navigation), the URL/params were stripped via `history.replaceState`, and
the "Login didn't complete" notification carried the signal. Over the
observation's full 60 s window: zero unauth/401/403/auth-5xx rows and zero
failed ws handshakes — nothing re-fires the exchange, so the limiter budget is
no longer burning (pre-fix, a user-stuck spinner plus any reload loop would
exhaust the 5-per-15 s budget into 429s; draft issue (d) below).

## hung — watchdog converges, then the late success recovers (watchdog + late-success redirect closed)

`DELAY_AUTH_MS=15000`, past the 12 s watchdog. Pre-fix reality: no
watchdog, one continuous spinner for 15.6 s, then the late 200 converged
normally (slow success).

Post-fix run (batch 1, final build) — the pre-fix arm SKIPped; all three
fix-dependent arms PASSED:

```
DOC    1790920382363  GET /?redeem_ready=true 200
LOADER 1790920382486  on     (doc+123 ms)   ← pre-armed pending: LoadingScreen immediately
LOADER 1790920394468  off    (doc+12105 ms) ← watchdog dropped the gate — 3040 ms BEFORE the late 200
req    1790920397508  POST /auth/login/ExchangeForJwt 200 (round-trip 15.043 s)
TOKENS 1790920397511  {"current":"6abf067c…"}   ← the LATE 200 still stored its token
LOADER 1790920397779 on / 1790920398080 off    ← late-success redirect transit
final URL: /  (late-success redirect: login page found the fresh flag + jwt and sent the user to /)
PASS post-fix: watchdog dropped the gate BEFORE the late 200 (loader-off observed pre-settlement)
PASS post-fix: zero-residual 60s window after the watchdog settlement (0; failed ws: 0)
PASS post-fix: watchdog converges (gate drops -> /login, then the late-success redirect recovers)
```

This is the full redeem-gate sequence working end-to-end, now with the pre-arm
engaged: the LoadingScreen is up from doc+123 ms (module-scope pre-arm), the
watchdog fired at doc+12105 ms and settled-failed the gate, the user landed on
`/login` instead of spinning, and when the exchange finally returned 200 three
seconds later, mogh_ui's own handler re-stored the token and the one-shot late-success
redirect (fresh `ok` flag + jwt present in the store →
`location.replace(safeLocalPath(backto))`, origin-guarded per round-8
C-001) recovered the login. Zero residual over the
60 s window. (Batch 0 had already passed this row on the pre-pre-arm build —
same three arms; the watchdog path was never pre-arm-sensitive, since the
mutation's own settlement handles both.)

## m1-seeded — read deferral: **FAIL on batch 0 → PASS on the final build** (m1 criterion MET)

### Batch 0 — the failure that found the defect (image `5190032c1dbe`, HEAD `9d9a38a88`)

Verbatim from the batch-0 run (exit 1):

```
PASS exchange 200 observed (pending window closed; ts=1790918209.054)
FAIL stale-token requests fire during the redeem window (wire: auth+rejected in [drive start -> exchange 200]; observed 2, statuses 500)
FAIL stale token VALUE confirmed on those requests (fetch shim tail …hwIjoxfQ.stale-signature)
PASS fresh token stored after the exchange (current pointer moved off stale-user)
```

Under `--post-fix` the first two predicates invert (demand ZERO stale rows / NO
stale-tail fetch). The ndjson timeline:

```
DOC    1790918207418  GET /?redeem_ready=true 200            ← redeem document loads
FETCH  1790918207516  /user               tail …stale-signature
FETCH  1790918207517  /read/GetCoreInfo   tail …stale-signature
REQ    1790918207520  GET  /user              500 (auth)         ← both rejected 500
REQ    1790918207520  POST /read/GetCoreInfo  500 (auth)
FETCH  1790918207522  /auth/login/ExchangeForJwt  tail …stale-signature   ← the redeem dispatch
LOADER 1790918207541  on                                          ← LoadingScreen engages AFTER the reads
200    1790918209054  ExchangeForJwt (1500 ms knob)
TOKENS 1790918209056  {"current":"6abf067c…"}                     ← fresh pointer (PASS)
```

What batch 0's ndjson proved (both sides, from this one file): the deferral IS
in force during actual pending — zero `/user|/read` fetches in
[exchange dispatch → 200] — but the two stale reads escape BEFORE the flip, with
the same shape as the pre-fix batch (reads at `…697471/472` vs dispatch `…477`).

### Root cause — wiring, not timing luck

1. mogh_ui arms the redeem synchronously in **Router's** render body
   (`useAuthState`: `if (jwt_redeem_ready && !jwt_redeem_sent) { redeemJwt({}); … }`).
2. The gated reads live in **WebsocketProvider — Router's parent** (`main.tsx`:
   `QueryClientProvider > WebsocketProvider > ThemeProvider > Router`).
   React renders parent-first, so the provider renders BEFORE the arm exists,
   with the gate still `idle` (open by design). Its `useUser`/`GetCoreInfo`
   observers commit with `enabled: true` and their mount fetches fire at
   +98 ms — ~6 ms before the exchange dispatch (+104 ms) flips the gate; the
   LoadingScreen row lands +119 ms (100 ms shim poll).
3. A render-phase flip inside a CHILD cannot cover an ANCESTOR's same-commit
   observers. The `router.tsx` comment ("onMutate … flips the gate to `pending`
   before the snapshot read below, so render #1 is already the LoadingScreen")
   was true for Router's own output only. This also corrects the pre-fix premise
   reading above: the stale reads never fired "during pending" — they fire in
   the same commit as the arm, pre-dispatch; the check window
   ([drive start → exchange 200]) simply includes that pre-arm gap.

### The fix — module-scope pre-arm (`641f966da`, reviewed)

Exactly the minimal remedy batch 0's analysis sketched: `redeem-gate.ts`
initializes `state = "pending"` at import (before any React render) when
`location.search` carries `redeem_ready=true`, and arms the watchdog from the
same point — mirroring mogh_ui's own module-level `jwt_redeem_sent` guard. The
provider's FIRST render already sees the gate closed; the later real `onMutate`
flip is idempotent (equality guard) and its watchdog arm a no-op (already set).
Normal loads (no param) stay `idle`, byte-identical. The pre-armed watchdog is
also defensive: if mogh_ui drift ever stopped firing the mutation, the window
still converges to `settled-failed` within `WATCHDOG_MS`.

Companion checker fix in the same commit (verify.mjs, one predicate): the m1
value arm's shim filter had matched ANY fetch row carrying the stale jwt tail
before the wire exchange ts — but the redeem request itself legitimately
carries the stale jwt (it IS the credential being exchanged), and the shim logs
that row at DISPATCH, necessarily before the wire completion ts. Once the real
leaks were closed, the exchange self-matched and false-failed the absence arm
(first post-pre-arm run: wire observed 0, yet the value arm FAILED on the
exchange dispatch row alone). The filter is now scoped to the same
`^/(user|read)` paths the wire arm scopes — pre-fix behavior unchanged,
post-fix absence meaningful.

### Batch 1 — PASS on the final build (image `28f89a5be8af`, HEAD `641f966da`)

```
PASS exchange 200 observed (pending window closed; ts=1790920316.824)
PASS stale-token requests fire during the redeem window (wire: auth+rejected in [drive start -> exchange 200]; observed 0, statuses -)
PASS stale token VALUE confirmed on those requests (fetch shim tail …hwIjoxfQ.stale-signature)
PASS fresh token stored after the exchange (current pointer moved off stale-user)
SCENARIO m1-seeded PASS   (exit 0)
```

The ndjson shows the pre-arm doing exactly what the root cause demanded: the
redeem document loads at `…315199`, the exchange dispatch fetch goes out at
`…315295` (doc+96 ms) — and **the exchange itself is the ONLY fetch in
[doc load → 200]**; the LoadingScreen row lands doc+117 ms; zero stale-tail
fetches to `/user|/read` anywhere before the 200. The scenario-map arm ("zero such
rows") holds on the final build; the m1 acceptance criterion is met.

### PRE-ARM PROOF — the parent-first gap fired in a second scenario (batch-0 `exchange-error`)

Independent confirmation of the root cause, from a run whose scenario the
earlier revision of this document believed untouched by the pre-arm change.
Batch 0's `exchange-error` ndjson (pre-pre-arm build): the replay document
dispatched auth-bearing `GET /user` + `POST /read/GetCoreInfo` BEFORE the
exchange — the fetch shim logged both at `ts_ms 1790918528200` against the
exchange dispatch at `…528202` (−2 ms), the wire shows their completions at
`1790918528.2028` / `…202812` against the 401 at `1790918528.2185` (−16 ms),
and Caddy pinned both rows to the redeem document
(`Referer: …/?redeem_ready=true`). Same shape as m1's leak: provider reads
committed while the gate was still open. They answered 200 (the leg-1 jwt was
still stored), so nothing rejected — the harm here is evidentiary, not budget:
the gap was real outside `m1-seeded`, and this document mis-assessed its own
evidence in claiming otherwise.

Batch 2 (final build) closes the loop in this same scenario: between the replay
document load (`1790942486.976`) and the exchange dispatch, the fetch shim
logged zero rows — the exchange is the document's first API call (see the
`exchange-error` section above).

---

## Upstream issue drafts (harness evidence attached; NOT filed)

DRAFT text for each upstream report. File paths verified against the pinned
dependency builds in `ui/node_modules` (`mogh_ui`, `mogh_auth_client` — the
versions stock v2.3.3 resolves).

### (a) mogh_ui / mogh_auth_client — `add_and_change` silently drops tokens whose `sub` is falsy

**Title:** `LOGIN_TOKENS.add_and_change silently discards a valid exchange response when the JWT has no `sub` claim`

**Body skeleton:**

- `mogh_auth_client/dist/tokens.js` (`add_and_change`):

  ```js
  const add_and_change = (jwt) => {
      const user_id = extractUserIdFromJwt(jwt); // jwtDecode(jwt).sub
      if (!user_id)
          return;                                 // silent: no throw, no log, no signal
  ```

- Failure shape (harness `m2-forced`, moghtech/komodo#1665 repro): the exchange
  returns upstream **200** with a structurally-valid jwt lacking `sub`;
  `add_and_change` drops it; the caller's `onSuccess` then treats login as
  complete and triggers the sanitize reload — the user lands on `/login` with
  **zero indication** anything failed (pre-fix evidence: every TOKENS row stays
  `null`, the jwt's payload never appears in storage, reload fetches the login
  chunk + `GetLoginOptions`).
- Expected: the drop is impossible to diagnose for the user and for support
  (looks like "login redirected me back and nothing happened").
- Suggested direction: throw/log on the falsy-`sub` early return (or surface a
  drop flag the host app can read). komodo's fix now detects + surfaces this
  client-side (`komodo-redeem` phase `drop` + a persistent toast), but the
  library behavior remains a silent swallow for any other embedder.

### (b) mogh_auth_client — `LOGIN_TOKENS` has no storage-event re-sync

**Title:** `LOGIN_TOKENS state is a module-init localStorage snapshot — no `storage` event listener, so multi-tab/multi-document state diverges`

**Body skeleton:**

- `mogh_auth_client/dist/tokens.js`: the IIFE reads `localStorage.getItem(...)`
  **once at module init** into a closure variable; every read (`jwt()`,
  `accounts()`) serves that closure; the only writer is the module's own
  mutators via `update_local_storage()`. No `window.addEventListener("storage",
  …)` anywhere in the module.
- Consequences: a second tab's login/logout/change never re-syncs this tab's
  closure (and vice versa); a long-lived document can keep sending a jwt that
  storage no longer contains (or miss one that was just stored).
- Harness evidence (adjacent, single-context suite): the redeem race fix had to
  re-read storage from `localStorage` directly to detect the (a) drop — the
  module's closure was not a trustworthy reflection of storage mid-login
  (`redeem-gate.ts` cites `tokens.js:5` for the key); and each new document
  re-reads the store at init (the suite's seed-once-per-context deviation:
  an unguarded re-seed re-poisoned every post-reload document — the per-document
  re-snapshot is observable), while a NOT-reloaded document never re-reads.
  Multi-tab divergence itself is outside this suite's reach (one browser
  context per drive) — flagged as code-verified, harness-adjacent.
- Suggested direction: subscribe to `storage` for `mogh-auth-tokens-v1` and
  re-hydrate the closure (or read through to storage per call).

### (c) mogh_ui — redeem failure has no in-document convergence (eternal spinner) + `sanitizeQuery` = full-page-reload control flow

**Title:** `Failed ExchangeForJwt leaves `redeem_ready=true` in the URL and the app on a LoadingScreen forever; success path uses a full-page reload as control flow`

**Body skeleton:**

- `mogh_ui/dist/auth/index.js` (`useAuthState`): the redeem mutation is armed
  from `search.get("redeem_ready") === "true"` with only an `onSuccess`
  (`useLogin`'s built-in `onError` shows a toast and logs — nothing else). On
  failure: no navigation, no param strip, no state change. Any host that renders
  a LoadingScreen while `jwt_redeem_ready` (as komodo's router did) spins
  **forever**: the URL keeps `redeem_ready=true`, and a manual refresh re-fires
  the exchange against the one-shot pending state (401, burning one of the
  5-per-15 s limiter attempts — see (d)).
- Harness evidence (`exchange-error`, pre-fix): replayed 401 → no navigation for
  the whole 8 s observation, last LoadingScreen state `on` indefinitely (M5);
  post-fix, komodo converges in-document (exactly one document load, zero
  residual over 60 s; the endpoint depends on what the store holds —
  `/login` with no surviving session, the app itself when a valid one
  survives, per round-8 C-003's store neutrality) — but the library still
  offers embedders no failure path.
- Success-path semantics (`mogh_ui/dist/auth/utils.js`, `sanitizeQueryInner`):
  control flow = delete `redeem_ready|totp|passkey` from the query +
  `location.replace(origin+pathname+query)` — a **full-page reload** on every
  successful login. That reload re-enters any forward-auth gateway in front of
  the app (see (e)), and it is why a failed-redeem spinner can't be fixed from
  inside the library without an in-document path.
- Suggested direction: an `onError` that strips the redeem params and navigates
  to the login route in-document; replace the sanitize reload with
  router-state updates where a router is available.

### (d) komodo core — auth rate limiter keys by IP, which locks out shared-egress deployments

**Title:** `Auth rate limit (5 attempts / 15 s) keyed by peer IP locks out all users behind one proxy egress`

**Body skeleton:**

- Observed on this harness (core `2.3.3`, `auth_rate_limit_max_attempts` unset →
  default 5): the wire answers a replayed exchange with `401 … You have 4
  attempts remaining` — failed attempts share one per-IP budget, and a client
  loop that re-fires the exchange exhausts the budget into 429s ("7 exchanges /
  0 authed" in moghtech/komodo#1665 is this shape). Harness evidence:
  `exchange-error`'s replay 401 visibly burns budget; the suite paces all
  scenarios ≥16 s apart for exactly this reason (README "Verify suite").
- Topology (the reporter's shape, mirrored by the harness Caddy front): an
  authenticating gateway terminates all client connections, so core sees ONE
  peer IP for every user — one user's failure loop (or one office's NAT) can
  lock out everyone behind that egress for the window.
- Suggested direction: key the limiter by something the gateway attests
  (`X-Forwarded-For` trust configuration, session id, or client identity), or
  make the key + trusted-proxy count configurable; document the interaction
  with `oidc_auto_redirect` loops (a redirect loop is exactly the failure mode
  that burns this budget fastest).

### (e) OPEN QUESTION — forward-auth session interplay with the post-exchange navigations (M3, mock-limited)

**Title (question form):** `Does a forward-auth gateway's session survive the OAuth token redemption instant — and which of the post-exchange navigations does it break?`

**Body skeleton (framed as a question, not a claim — the harness mock CANNOT
express this):**

- The harness mock's portal session **survives** token redemption, so every
  post-exchange navigation (mogh_ui's sanitize reload on success; komodo's
  late-success redirect; the failure-path convergence) re-enters the komodo vhost through the
  gate and returns 200 — M3's bounce/loop (forward-auth session consumed at the
  redemption instant → the post-exchange reload 302s to the portal → …) was
  **not reproducible** and no `/verify` 401/503 storm appears in any ndjson
  (BASELINE "M3 non-reproduction" above).
- Post-fix there are up to TWO navigations after a successful exchange (sanitize
  reload; plus the late-success redirect only when the watchdog path was taken) and ONE
  in-document convergence on failure (no reload). If a production Authelia-class
  gateway invalidates its session at the redemption instant, which navigation
  bounces first, and does the loop re-arm the redeem (the URL still carrying
  `redeem_ready=true` through the portal bounce) — re-burning the one-shot and
  the (d) limiter budget?
- Ask for upstream: confirmation of the intended gateway contract at the
  redemption instant, so the client-side failure paths can be designed against
  reality rather than the mock's permissive behavior.

### (f) mogh_ui — `backto` query param flows unvalidated into `location.replace`

**Title:** `Open redirect: the `backto` query param is passed to `location.replace` without any origin validation`

- Code-verified (no harness scenario drives a hostile `backto`):
  `mogh_ui/dist/auth/login/index.js` `maybeNavigate` →
  `location.replace(new URLSearchParams(location.search).get("backto") ?? "/")`
  — any absolute URL in `backto` navigates the user off-site after login.
  Komodo's own late-success call site now guards locally (round-8 C-001:
  local-path-only + control-character rejection — note the parser strips
  tab/newline BEFORE parsing and folds `\` to `/` for special schemes, so
  naive second-character guards are bypassable via `/%09//evil.example`;
  validating the raw string certifies a string the parser never sees), but
  mogh_ui's `maybeNavigate` remains unvalidated for every embedder.
- Ask: same-origin validation (or relative-path-only) upstream, so every
  embedder inherits the fix — komodo's local guard is compensation, not the
  fix.

### One-line ask (not drafted)

- mogh_ui: consider exporting the token-store key and redeem lifecycle seams so
  hosts can unit-test redeem integrations without a full harness (store shape:
  `{ current, tokens: [{ user_id, jwt }] }`).

---

*Metadata (post-fix section): produced by the post-fix runs — batch 0 2026-10-02
02:14–02:25 -03:00 (HEAD `9d9a38a88`, image `5190032c1dbe`, verify.mjs as
committed at `fa53da3c0`); batch 1 (final) 02:51–02:55 -03:00 (HEAD
`641f966da`, image `28f89a5be8af`, verify.mjs with `641f966da`'s one-predicate
m1 scoping fix); batch 2 08:58–09:04 -03:00 (same HEAD
`641f966da`, same image `28f89a5be8af` — id re-verified on the running
container before the batch; the re-run rows are `latency` 08:58, `m2-forced`
09:00, `exchange-error` 09:01, each ≥16 s apart); round-8 batches 3a/3b —
3a ~16:50 -03:00 (HEAD `e6192cf37`, image `8835351d67ac`, 6/7 — m2-forced
FAIL, root-caused to the doomed-document consume race, fixed in
`dfa654392`), **3b canonical 17:08–17:14 -03:00 (HEAD `a861602a4`, image
`778aee5742c6`, 7/7, every ndjson meta row stamped
`head=a861602a4279, dirty=0`)**; round-9 batches 4a/4b — 4a ~17:57 -03:00
(HEAD `78739f1fc0f8`, 7/8 — the new ws-alive check mis-failed a healthy run
on a wrong ts-model predicate; root-caused by an instrumented diagnostic and
fixed in `1f0f2673b0bb`), **4b canonical ~18:29–18:43 -03:00 (HEAD
`1f0f2673b0bb`, image `925a844ef0a0`, 8/8, every ndjson meta row stamped
`head=1f0f2673b0bb, dirty=0`)**; round-10 batch 5 canonical ~21:20–21:27
-03:00 (HEAD `9bf2f4a26`, image `c30be6e0dc43`, 8/8, every ndjson meta row
stamped `head=9bf2f4a26fd7, dirty=0`; served-bundle identity verified by
md5 against the host build); round-11 batch 6 canonical ~09:47–09:59
-03:00 (HEAD `7ee624c16`, 8/8, every ndjson meta row stamped
`head=7ee624c168f7, dirty=0`; served-bundle md5 verified; the harness's
own Playwright driver now pinned 1.63.0 with a committed lockfile).
Harness per README "Run"
(podman shim, `compose/oidc-dev.compose.yaml`, `oidc-provider@9.12.2`); raw
ndjson evidence in gitignored `compose/oidc-dev/out/` (batch 3b files are
the surviving evidence for all seven rows); run stdout
archived by the task outside the repo. All batch 0–2 commit hashes are
pre-rewrite; full history on the fork's
`komodo-oidc-login-jwt-redeem-race-fix-1665-archive` branch.*
