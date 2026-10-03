# OIDC redeem-race harness (moghtech/komodo#1665)

Repro harness for the OIDC-login JWT-redeem race: mongo + komodo-core serving the
**branch-built UI** from `/app/ui` exactly like production, the OIDC
mock (`oidc-mock`) with its forward-auth `/verify` endpoint, the Caddy front
(`komodo.oidctest.localhost` + `portal.oidctest.localhost`, forward_auth gate,
JSON access log) and the `delay` sidecar (latency knob for `/auth/login/*`).

## Run

Create `compose/oidc-dev/.env` first — it doesn't exist until Step 1 pins the digest.

```sh
# Bind-mounted FILE sources must exist BEFORE the first `up` (a missing bind
# source is pre-created as a DIRECTORY, which breaks Caddy's file logger and
# turns the later CA export into "Is a directory" errors).
touch compose/oidc-dev/access.log compose/oidc-dev/caddy-root-ca.crt
# If a previous `up` already materialized either source as a directory:
rmdir compose/oidc-dev/access.log compose/oidc-dev/caddy-root-ca.crt 2>/dev/null

docker compose -f compose/oidc-dev.compose.yaml --env-file compose/oidc-dev/.env up -d --build
docker compose -f compose/oidc-dev.compose.yaml ps
curl -s -o /dev/null -w '%{http_code}\n' http://localhost:9120   # expect 200
```

**First boot ordering for the CA** (only when `oidc-dev-caddy-data` volume is
fresh): Caddy generates its internal CA on first TLS use. Export the root into
core's trust **after** the first `up`, then recreate core:

```sh
docker compose -f compose/oidc-dev.compose.yaml --env-file compose/oidc-dev/.env \
  exec caddy cat /data/caddy/pki/authorities/local/root.crt \
  > compose/oidc-dev/caddy-root-ca.crt
docker compose -f compose/oidc-dev.compose.yaml --env-file compose/oidc-dev/.env up -d core
```

`caddy-root-ca.crt` is gitignored (it is derived state — a fresh volume makes a
new CA and a stale committed file would silently break core's discovery fetch).
Core reads it via `SSL_CERT_FILE=/config/caddy-root-ca.crt` (honored by
rustls-native-certs / openssl-probe). Symptom when missing/stale/empty: core
logs `invalid peer certificate: UnknownIssuer` on `/auth/oidc/login` and
returns 500 — until the export + `up -d core` recreate below.

> On hosts where `docker` is a podman shim (no compose plugin), use the standalone
> binary against the podman socket: `docker-compose -f … --env-file … up -d --build`
> with `DOCKER_HOST=unix:///run/user/$UID/podman/podman.sock`.
>
> **Podman ≤ 3.4 bootstrap gotcha:** compose writes the project network's conflist
> with `cniVersion: 1.0.0`, which podman 3.4's CNI stack refuses to load
> ("CNI network not found" at container start). First `up` creates the network and
> fails; patch `~/.config/cni/net.d/komodo-oidc-dev_default.conflist`
> (`cniVersion` → `0.4.0`) once, then re-run `up`. Only needed on fresh network
> creation — `down` removes the network, so expect the patch again after it.

## Step 1 — pin the core image digest

`compose/oidc-dev/.env` holds `KOMODO_CORE_IMAGE`, the digest-pinned base image that
`core-ui.Dockerfile` builds on (`FROM ${CORE_IMAGE}`). It is gitignored — never commit it.

To re-pin (e.g. after a version bump), resolve the tag to a manifest digest:

```sh
# Note: the tag has NO leading "v" — ghcr.io/moghtech/komodo-core:2.3.3, not :v2.3.3.
docker buildx imagetools inspect ghcr.io/moghtech/komodo-core:2.3.3 | grep -i digest | head -1
```

No buildx? The registry API works the same way (anonymous pull on ghcr):

```sh
TOKEN=$(curl -s "https://ghcr.io/token?scope=repository:moghtech/komodo-core:pull" \
  | sed -n 's/.*"token":"\([^"]*\)".*/\1/p')
curl -sI -H "Authorization: Bearer $TOKEN" \
  -H "Accept: application/vnd.oci.image.index.v1+json,application/vnd.docker.distribution.manifest.list.v2+json" \
  https://ghcr.io/v2/moghtech/komodo-core/manifests/2.3.3 \
  | grep -i docker-content-digest
```

Write the result into `compose/oidc-dev/.env`:

```sh
KOMODO_CORE_IMAGE=ghcr.io/moghtech/komodo-core@sha256:<digest>
```

Current pin (set 2026-10-01, tag `2.3.3`, OCI index digest):

```
sha256:bca73d0eee143066228fb9b80c38a5a2726e573ad2758f7aa92b5c91a8aef1c7
```

## Pinned dependencies

- **`node-oidc-provider` version `9.12.2`** — the latest release at harness time.
  The plan draft pinned "`11.10.1`", but that version does not exist on npm (no
  11.x line was ever published; latest = 9.12.2, previous = 6.31.1 — verified
  against the registry 2026-10-01). The API notes in `oidc-mock/index.mjs` are
  written against the 9.12.2 **source in `node_modules`**, not docs from memory.
  Bump only with a re-run of the full suite.

## Verify endpoint implementation note

`/verify` runs on a bare `node:http` server in front of the provider. Getting
`provider.Session.get` to work on a raw request took three source-verified
adaptations (all against `oidc-provider@9.12.2`, verified empirically after
boot — see acceptance evidence in the task):

1. **Pass `{ req, res }`, not the raw `req`.** `Session.get(ctx)` supports two
   shapes (`lib/models/session.js`): a Koa ctx with `ctx.oidc`, or any object
   with `.req`/`.res` — it calls `provider.createContext(ctx.req, ctx.res)`
   (Koa inherits `createContext`; `Provider extends Koa`). A bare
   `IncomingMessage` has neither shape, so `Session.get(req)` would throw →
   every request 503.
2. **Check `session.accountId`, there is no `userId()`.** The plan draft called
   `session.userId()`, which does not exist on the Session model — it would
   throw a TypeError into the 503 path on every call. `accountId` is the
   persisted login field.
3. **`provider.callback` is Koa's handler *factory*.** `Provider extends Koa`,
   so `provider.callback()` returns the `(req, res)` handler; calling
   `provider.callback(req, res)` would build a handler and drop it (requests
   hang). Build it once and reuse.

Semantics: missing cookie → `Session.get` returns an empty Session (it does not
throw), `accountId` undefined → **401**; adapter/lookup failure → catch →
**503**; Caddy's access log can therefore distinguish "no login yet" from
"provider broken". A forged cookie value 401s (empty session), verified.

## Driving a login with curl (what the Playwright suite will automate)

End-to-end verified flow (2026-10-02) through the full Caddy front. One cookie jar
carries everything: this curl (7.81) DOES accept the parent-domain `_session`
cookie into the jar (the Cookie-caveat note below about refusing it does not
reproduce here), so no manual `-H "Cookie: …"` stitching is needed. Steps
1–4 share the jar or the login POST dies with SessionNotFound.

The login has TWO phases. Phase A establishes the **portal session** via the
`/verify` gate's redirect (the whole komodo vhost is gated, so even
`/auth/oidc/login` 302s out until a portal session exists — mirroring the
reporter's gateway shape). Phase B is the real komodo OIDC login, which
auto-resumes without a second login POST because the session already exists.

```sh
K=https://komodo.oidctest.localhost:8443; P=https://portal.oidctest.localhost:8443
R="--resolve komodo.oidctest.localhost:8443:127.0.0.1 --resolve portal.oidctest.localhost:8443:127.0.0.1"
J=jar; rm -f $J
loc() { grep -i '^location' "$1" | sed 's/^[Ll]ocation: //; s/\r//'; }

# -- Phase A: portal session via the gate's redirect ---------------------
# 1. unauthenticated / → 302 to the throwaway authorize (redirect_uri=$P/dev)
L1=$(curl -sk $R -o /dev/null -D h1.txt "$K/"; loc h1.txt)
# 2. → 303 /interaction/:uid; 3. login POST (any credentials) → resume;
curl -sk $R -o /dev/null -D h2.txt -b $J -c $J "$L1"; IUID=$(loc h2.txt)
curl -sk $R -o /dev/null -D h3.txt -b $J -c $J -d 'prompt=login&login=alice&password=x' "$P$IUID"
RESUME=$(loc h3.txt)
# 4. resume → 303 $P/dev?code=… and _session is set (consent auto-skipped)
curl -sk $R -o /dev/null -D h4.txt -b $J -c $J "$RESUME"

# -- Phase B: the real komodo OIDC login --------------------------------
# 5. /auth/oidc/login → 302 to the REAL authorize: note redirect_uri=
#    $K/auth/oidc/callback and komodo's own code_challenge (S256, minted
#    by core). Requested scope arrives as "openid openid profile email"
#    (komodo prepends openid to a base list — duplicate is accepted by v9).
curl -sk $R -o /dev/null -D h5.txt -b $J -c $J "$K/auth/oidc/login?redirect=%2F"
L5=$(loc h5.txt)
# 6. authorize auto-resumes (session present, no prompts) → 303 komodo callback
curl -sk $R -o /dev/null -D h6.txt -b $J -c $J "$L5"; CB=$(loc h6.txt)
# 7. callback → 303 /?redeem_ready=true   (the expected landing)
curl -sk $R -o /dev/null -D h7.txt -b $J -c $J "$CB"; loc h7.txt
# 8. UI reachable through the gate (parent-domain cookie works)
curl -sk $R -b $J "$K/?redeem_ready=true"      # 200, <title>Komodo</title>
# 9. the redeem: ExchangeForJwt takes an EMPTY body (pending state is
#    server-side); first call → 200 {"jwt": "..."}
curl -sk $R -X POST -H 'content-type: application/json' -d '{}' -b $J \
  "$K/auth/login/ExchangeForJwt"
```

### PRE-FIX OBSERVATION (wire-level baseline feeding BASELINE.md, recorded 2026-10-02)

- **First `ExchangeForJwt` → 200 with `{"jwt": …}`.** Empty request body — the
  pending login (incl. PKCE verifier) is server-side state.
- **Replaying `ExchangeForJwt` → 401** `Authentication steps must be completed
  before JWT can be retrieved | You have 4 attempts remaining`. The pending
  state is ONE-SHOT, and core's `auth_rate_limit_max_attempts: 5` counts the
  burn — a UI loop that re-fires the exchange exhausts 5 attempts then most
  likely 429s. This is the closest wire-level analogue to the reporter's
  "7 exchanges / 0 authed".
- **The sanitize reload does NOT bounce** with this mock: a second
  `GET /?redeem_ready=true` right after the exchange → 200. The mock's portal
  session SURVIVES token redemption, so the M3 "forward-auth consumed the
  session at that instant → redirect loop" theory does NOT reproduce against
  this mock by itself (M1/M2/M5 stay live; do not log a reproduced M3 from a
  loop seen here).
- **Follow-up requests carry no authorization at the wire** (curl attaches
  none), and core 401s an unauthenticated `POST /api/GetVersion`. Whether the
  browser attaches the JWT it received is the client-side question the fix
  addresses — curl cannot exercise mogh_ui's JS.
- Debugging discipline note: the access log's two `500 GET /auth/oidc/login`
  entries were **core discovery TLS failures** (`invalid peer certificate:
  UnknownIssuer`, before the CA export above) — a lookup error in the chain,
  exactly the class of confounder that must NOT be recorded as a reproduced M3. Check
  `docker compose logs core` FIRST when the login 500s; `/verify` 503 (mock
  lookup error) has not occurred yet.

The direct-to-mock flow (loopback :3344, `code → POST /token` with
`client_secret_basic` + PKCE S256 → id_token `sub=alice` / `aud=komodo-harness`,
`GET /me` → `{"sub":"alice","email":"alice@oidctest.local"}`) remains valid —
userinfo is the `/me` route in v9, and komodo reads the profile from userinfo
because the id_token carries only `sub`. Use `$P/dev` or the komodo callback
as redirect_uri exactly as registered (see `oidc-mock/index.mjs`); the URIs
now carry `:8443`.

> **Caveat (resolved):** discovery endpoints are built from the
> *request* origin, not the issuer config — source-verified in v9.12.2
> (`helpers/oidc_context.js` `urlFor` → `this.ctx.href`; only the
> discovery document's `issuer` field stays pinned to `MOCK_ISSUER`). Two
> consequences live in the mock: `provider.proxy = true` (Koa then takes
> scheme/host from Caddy's `X-Forwarded-Proto`/`-Host` — otherwise every
> discovered URL derives as `http://…` while the issuer is https), and Caddy
> serves the portal vhost so the browser-facing endpoint URLs carry the portal
> origin. The `issuer` field pinning is why `oidc_provider` in
> `core-config.toml` must be byte-identical to `MOCK_ISSUER`.

## Cookie caveat

The mock sets its session cookie on the parent domain `oidctest.localhost` — required
so that Caddy's `forward_auth` subrequest (originating at `komodo.oidctest.localhost`)
can see the portal's cookie. Host-only cookies for `portal.oidctest.localhost` would
401 every komodo request, including `/auth/oidc/callback`, and no login could ever
complete. If a test browser rejects `Domain=` attributes under `.localhost` (public
suffix list edge), switch the harness hosts to a `*.oidctest.test` style name plus
`/etc/hosts` entries — do **not** drop the parent-domain cookie.

## Deviations from the original sketches (OIDC-enable reality-checks)

The original OIDC-enable snippets predate the Caddy-front and direct-to-mock
findings; where reality differed, reality won and is documented here.

1. **Published port is `127.0.0.1:8443 → 443`, not `80:80` + `443:443`.** The
   host's system Caddy already owns 80, and rootless podman cannot publish
   privileged ports at all (`rootlessport cannot expose privileged port 443 …
   choose a larger port number (>= 1024)`; `ip_unprivileged_port_start=1024`).
   No `[::1]` twin either: rootlessport cannot bind IPv6 loopback, and the host
   resolver answers `*.localhost` with `127.0.0.1` only. **Every
   browser-facing origin therefore carries `:8443`** — `MOCK_ISSUER`,
   `KOMODO_ORIGIN`, core `host`/`KOMODO_HOST`, `oidc_provider`, and the mock
   client's `redirect_uris`. Loopback-only binding is the security boundary
   for the portal vhost (the mock behind it auto-consents anyone), same as the
   mock's `127.0.0.1:3344`.
2. **Caddy listens on container ports 443 AND 8443.** The network alias
   (`portal.oidctest.localhost` on the caddy service) fixes NAME resolution
   from inside the compose network, but core dials the issuer URL's PORT —
   `https://portal.oidctest.localhost:8443` → caddy container port 8443. With
   only `:443` listeners core got `tcp connect error: Connection refused
   (os error 111)` on discovery.
3. **Core trusts Caddy's internal CA via `SSL_CERT_FILE`.** Core's HTTP stack
   is reqwest+rustls; first `/auth/oidc/login` failed with `invalid peer
   certificate: UnknownIssuer` until Caddy's root was exported (command under
   Run) and bind-mounted into core. `SSL_CERT_FILE` is honored (openssl-probe),
   so no image rebuild or CA-bundle overwrite is needed.
4. **`forward_auth` lives INSIDE the `route` block — for written-order
   determinism, not as a bypass fix.** Caddy's directive order already sorts
   `forward_auth` (middleware group) BEFORE `handle`/`route`/`reverse_proxy`;
   verified empirically by adapting the same site without the route block —
   the gate still compiled first. Wrapping the site in one explicit `route`
   pins the evaluation order to the written order (verify-proxy → delay route
   → core route, per `caddy adapt`), immune to directive-order-table changes
   across Caddy versions.
5. **`/verify` redirects unauthenticated NAVIGATIONS to the portal** (the harness
   requires a 302 to the portal, not a bare 401). Navigation is detected by
   `X-Forwarded-Method` (Caddy's forward_auth subrequest always sets it):
   GET/HEAD → `302` to a throwaway authorize URL; everything else — and any
   direct probe WITHOUT `X-Forwarded-Method`, i.e. the direct-to-mock
   `curl /verify` recipe — still gets `401`. The throwaway authorize uses
   `redirect_uri=$PORTAL/dev` (a registered second URI): a gateway-initiated
   authorize carries a fixed PKCE challenge that could never complete komodo's
   exchange, so the landing must NOT be komodo's callback. Its only job is
   parking the user on the dev-interaction login to establish the portal
   session.
6. **`roll_disabled` on the access log** — Caddy's size rotation renames the
   file, which cannot work on the single bind-mounted FILE
   (`./oidc-dev/access.log`). The bind file must exist before the first `up`
   (missing bind sources are pre-created as directories).
7. **`oidc_redirect_host` stays empty.** Verified: komodo builds the
   authorization redirect from `host` (`KOMODO_HOST`) — the observed
   redirect_uri was `https://komodo.oidctest.localhost:8443/auth/oidc/callback`,
   matching the mock's registration without touching `oidc_redirect_host`.
8. **Mock sessions are in-memory and die on ANY stack reconcile** (`docker
   compose up` may recreate containers even for unrelated config changes —
   observed: an `up -d caddy` recreated mongo/core/mock and wiped the session).
   Re-drive Phase A after any `up`. The quick-start signing keys regenerate
   with the process, so old codes die too (harmless — nothing persists).

## Latency knob

```sh
DELAY_AUTH_MS=1500 docker compose -f compose/oidc-dev.compose.yaml \
  --env-file compose/oidc-dev/.env up -d delay   # recreate with a delay
docker compose -f compose/oidc-dev.compose.yaml --env-file compose/oidc-dev/.env \
  up -d delay                                    # back to 0
```

Only `/auth/login/*` is routed through the sidecar (everything else proxies
straight to core — measured 4ms with a 1500ms knob active). Keep the value
under the fix's 12 s watchdog; the dedicated past-watchdog harness row
uses 15000. Verified: 0.019 s baseline vs 1.53 s with `DELAY_AUTH_MS=1500`.
The scenarios manage this knob themselves (read via `printenv` inside the
delay container, recreated when it differs, restored to 0 afterwards) — run
the suite, not the knob.

## Verify suite — `verify.mjs`

One script drives the scenario matrix (see the row→scenario map below) over the Caddy JSON access log
plus browser instrumentation, prints per-assertion PASS/FAIL lines, then
`SCENARIO <name> PASS|FAIL` and exits with the matching code.

```sh
# one-time, dev-only HOST install — `npm ci` honors the COMMITTED lockfile
# (the driver is evidence machinery; its version is pinned, not floating):
cd compose/oidc-dev && npm ci && npx playwright install chromium

node compose/oidc-dev/verify.mjs success          # exit 0 = PASS
node compose/oidc-dev/verify.mjs latency          # sets DELAY_AUTH_MS=2000 itself, restores 0
node compose/oidc-dev/verify.mjs m1-seeded        # sets DELAY_AUTH_MS=1500 (widens the M1 window), restores 0
node compose/oidc-dev/verify.mjs m2-forced
node compose/oidc-dev/verify.mjs m2-seeded        # same sub-less exchange with a pre-existing session seeded (round-9 C-003 population)
node compose/oidc-dev/verify.mjs exchange-error   # ~90 s: 60 s residual window is OBSERVED, not assumed
node compose/oidc-dev/verify.mjs hung             # sets DELAY_AUTH_MS=15000, ~2.5 min
node compose/oidc-dev/verify.mjs isolation
node compose/oidc-dev/verify.mjs all              # sequential, 16 s gap between rows (per-IP auth limiter: 5/15 s)

# Post-fix runs only — also enforces the FIX-DEPENDENT assertions:
node compose/oidc-dev/verify.mjs <scenario> --post-fix
```

Notes:

- Requires the stack up (Run section). The harness host serves everything on
  `:8443`; the drive is two-phase (the whole komodo vhost is gated, so the
  first `/auth/oidc/login` entry 302s to the portal's throwaway authorize, and
  the SECOND entry auto-resumes — same shape as the curl recipe above).
- **Cookie-domain probe (the README's "Cookie caveat", settled
  2026-10-02): headless Chromium ACCEPTS the mock's parent-domain
  `Domain=oidctest.localhost` `_session` cookie — the full login completes and
  the cookie reaches the komodo origin. The `*.oidctest.test` + /etc/hosts
  fallback is NOT needed.**
- Every scenario writes `out/<scenario>.ndjson` (gitignored): request rows
  (`kind:"req"` — url, auth-present, status, ts, duration_s) and console-shim
  rows (`kind:"tokens"|"loader"|"fetch"|"ws-login"`), plus one `meta` row.
- Checks FAIL CLOSED: a check that cannot find its evidence throws (printed as
  an error line, verdict FAIL). FIX-DEPENDENT checks are `SKIP`ped (never
  PASSed) unless `--post-fix` is given — pre-fix runs must PASS on the
  mechanism-reproduction assertions alone.

### Access-log field notes (what the assertions actually rely on)

| Field | Reality (Caddy 2.11, this stack) |
|---|---|
| `ts` | **EPOCH-SECONDS float** — never `Date.parse` it; all window math stays in log-unit seconds (`logSec(e)`), cross-source comparisons multiply by 1000 onto the `Date.now()` timeline (same host clock). |
| `status` | Top-level **integer** per row (`resp_headers.Status` not used). The ws upgrade row carries an explicit `101`. |
| `duration` | Seconds, request start → response written. The exchange round-trip assertion (`latency`/`hung`) reads this — `duration × 1000 ≥ DELAY_AUTH_MS`. |
| `request.headers` | Nested map of arrays. **Caddy 2.11 redacts the VALUES of `Authorization` AND `Cookie` (`"[REDACTED]"`)** — presence is visible, values are not. This deviates from the earlier task note that authorization values are visible; value-level evidence (WHICH token was attached — the M1 discriminator) therefore comes from the browser-side `fetch` shim (`kind:"fetch"` rows carry the auth-header tail-24 fingerprint), while the wire rows prove presence + status + timing. |
| `resp_headers` | Header map DIRECTLY (no nested `.headers` key — unlike `request.headers`). `Location` is visible (not redacted). |
| ws rows | The `/ws/update` row is written when the connection **CLOSES**, not at the handshake — its `ts` must never anchor a time window (the success scenario closes the browser context to flush the row before checking). |
| hosts | Row hosts carry `:8443` (`komodo.oidctest.localhost:8443`). |

### Deviations from the original verify-suite sketch (reality won)

1. **The `m1-seeded` scenario manages the knob (`DELAY_AUTH_MS=1500`).** The
   sketch seeded the stale token and hoped the requests landed inside the
   ~100ms pre-fix window; the knob exists to "widen the race
   deterministically instead of relying on lucky timing". With 1500ms the
   stale-token `/user` + `/read/GetCoreInfo` requests reproducibly fire during
   pending. Restored to 0 after the row.
2. **Core answers a bad-signature jwt with `500`, not `401`** (observed on
   `GET /user` and `POST /read/GetCoreInfo`, 2026-10-02). The original "the server
   401s it" assumption is wrong; the M1 discriminating signal is the
   stale-token-authenticated request inside the window (plus the fetch-shim
   value proof), so the wire check accepts 401/403/5xx and prints the observed
   statuses.
3. **The seed is once-per-context, not per-document.** `addInitScript` runs on
   EVERY document; an unguarded seed re-poisoned the store after the
   post-exchange reload, so the dashboard's own reads came back stale-token.
   The seed now skips when a store already exists.
4. **`hung` pre-fix is NOT an eternal spinner at `DELAY_AUTH_MS=15000`** — the
   original design assumed it. Reality: mogh_ui has no timeout, so
   the LoadingScreen holds for the full 15 s (asserted: loader-on with **no
   OBSERVED gap** from landing to the late 200 — the shim polls at 100 ms, so
   sub-100 ms transitions could hide), then the late 200 stores the token and
   the sanitize reload lands the dashboard — a **slow success**. The eternal
   spinner only exists for an exchange that NEVER settles, which a finite delay
   sidecar cannot produce; that arm belongs to the post-fix watchdog
   (`--post-fix` asserts the gate dropped BEFORE the late 200 and the
   late-success redirect's landing). Pre-fix the row asserts what stock behavior actually guarantees.
5. **The success rows close the browser context before checking the ws row**
   (field note above — the 101 row lands at connection close). The
   browser-side `on_login` console line is the second, independent signal.
6. **`isolation` passes PRE-FIX** — stock v2.3.3 already contains a failing
   non-redeem execute (`POST /execute/StartDeployment` → 500) without a gate
   flip or reload. The scenario stays as the regression guard for the gate's
   filtered subscription (an unfiltered MutationCache listener would flip the
   gate here).
7. **`exchange-error`'s residual window excludes the settlement row itself**
   (strict `> settlement`): the replayed exchange carries the still-stored jwt,
   so counting the settlement 401 would fail the post-fix run on the correct
   implementation's own row. Pre-fix the residual count is printed as an
   observation and only ENFORCED under `--post-fix`.
8. The sketch's `check("... ", () => true)` stubs are gone: every check reads
   captured evidence (a check whose evidence is missing throws → FAIL).

### Spec row → scenario map

| Spec row | Scenario | Pre-fix assertion (mechanism) | Post-fix assertion (`--post-fix`) |
|---|---|---|---|
| Exchange 200, token stored | `success`, `latency` | exchange 200; authed follow-up ≤3 s; ws 101 + on_login; dashboard rendered; window total ≤4 & sliding ≤4 (+ round-trip ≥ delay in `latency`) | unchanged — these are the regression guard (green pre-fix) |
| Exchange 200, token dropped (M2) | `m2-forced`, `m2-seeded` | upstream 200 but sub-less jwt never stored; reload lands `/login` (`m2-seeded`: with a pre-existing session — the population the drift classifier once misread as key-drift) | drop flag surfaced in-document with `phase === "drop"` (strict — an unconsumed `ok` here is a classifier regression, not a pass); seeded row also asserts the prior session survives (store neutrality) |
| Exchange error (429 / consumed) | `exchange-error` | replayed exchange 401; no navigation for 8 s; LoadingScreen still up (eternal spinner, M5) | in-document convergence (exactly 1 document load), leg-1 session PRESERVED (≥1 authed `/user` 200 post-settlement; final `/` — round-8 C-003 store neutrality), zero-residual 60 s window |
| Hung exchange | `hung` | round-trip ≥ 15000 ms; LoadingScreen up with no OBSERVED gap until the late 200 (100 ms poll); slow-success reload documented | watchdog dropped the gate pre-settlement (loader-off before the 200); final landing `/` via the late-success redirect |
| No token, direct `/` | covered by the drive shape | every scenario's pre-session gate bounce (302 → portal) is asserted implicitly by the drive reaching the interaction page | unchanged |
| Stale token during redeem (M1) | `m1-seeded` | stale-token authed requests fire inside `[drive start → exchange 200]` (wire: auth+rejected; value: fetch-shim tail) | zero such rows (read deferral); fresh `current` pointer observed |
| Latched errors after settlement | (no dedicated row) | post-reload remount clears latched state implicitly — `success`'s dashboard + ws + authed reads assert the recovered state | unchanged |
| Non-redeem mutation unaffected | `isolation` | failing execute (500) → no reload, no LoadingScreen flip within 2 s (passes pre-fix — deviation 6) | unchanged — guards the gate's subscription filter |

### PRE-FIX BASELINE SUMMARY (stock v2.3.3 UI, observed 2026-10-02)

> **Superseded numbers — `BASELINE.md` is canonical.** This summary carries
> the verify suite's first-pass values (e.g. latency 2029 ms); the fresh
> baseline batch re-measured them (BASELINE.md header) and the post-fix run
> appended the post-fix half. Read `BASELINE.md` for both.

Feeds the pre-fix baseline (`BASELINE.md`):

- **`success`** — green (environment row): exchange 200, authed follow-ups, ws
  101 + on_login, dashboard; window bound 0 rows.
- **`latency`** — green; observed round-trip 2029 ms ≥ 2000 ms.
- **`m1-seeded`** — **M1 REPRODUCED**: `/user` + `/read/GetCoreInfo` fired with
  the seeded stale token during the pending window (fetch-shim value proof;
  core rejected with 500); fresh token stored after the exchange.
- **`m2-forced`** — **M2 REPRODUCED**: upstream 200 with a sub-less jwt →
  `add_and_change` silently drops it (no TOKENS transition), reload lands on
  `/login` with no signal.
- **`exchange-error`** — **M5 REPRODUCED**: replayed `/?redeem_ready=true` →
  exchange 401 → no navigation (URL keeps `redeem_ready=true`) and the
  LoadingScreen never clears (eternal spinner).
- **`hung`** — spinner holds the full 15013 ms (no OBSERVED gap at 100 ms
  poll granularity), then the late 200 converges (slow success). Pre-fix has
  no watchdog; the "eternal" arm is only reachable with a never-settling
  exchange.
- **`isolation`** — green pre-fix (gate already isolated; regression guard).

## Layout

| File | Purpose |
|------|---------|
| `../oidc-dev.compose.yaml` | The harness project: mongo + core + oidc-mock + caddy + delay |
| `core-ui.Dockerfile` | Stage 1 = `ui/Dockerfile`'s builder on this branch; stage 2 = digest-pinned core image + `COPY --from=builder /builder/ui/dist /app/ui` |
| `core-config.toml` | Mounted read-only at `/config/config.toml`; field names validated against `config/core.config.toml`; SINGLE source of truth for the OIDC toggles (env vars would override it) |
| `Caddyfile` | TLS front: portal vhost (plain proxy) + komodo vhost (forward_auth gate → delay/core split, JSON access log incl. request headers) |
| `delay.mjs` | The latency knob's sidecar (real awaited `setTimeout`, plain HTTP pipe) |
| `oidc-mock/` | The OIDC provider (`oidc-provider@9.12.2`, in-memory adapter, dev interactions) + `/verify` forward-auth endpoint + `/dev` landing on :3344 |
| `verify.mjs` | Playwright scenario suite: drives the login, tails the access log, prints per-assertion PASS/FAIL + `SCENARIO <name> PASS|FAIL`; writes `out/<scenario>.ndjson`. See "Verify suite" above |
| `out/` | LOCAL ONLY, gitignored — per-scenario ndjson evidence |
| `node_modules/`, `package.json`, `package-lock.json` | LOCAL ONLY, gitignored — the dev-only host playwright install |
| `access.log` | LOCAL ONLY, gitignored — Caddy's JSON access log; `touch` before first `up` |
| `caddy-root-ca.crt` | LOCAL ONLY, gitignored — exported from the caddy volume; re-export after volume recreation |
| `.env` | LOCAL ONLY, gitignored — `KOMODO_CORE_IMAGE` digest pin |
