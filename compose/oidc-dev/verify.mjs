#!/usr/bin/env node
// Scenario driver for the OIDC redeem-race harness (moghtech/komodo#1665;
// scenario matrix documented in README "Verify suite").
//
// Usage:
//   node verify.mjs <scenario> [--post-fix]
//     scenario: success | latency | m1-seeded | m2-forced | m2-seeded |
//               exchange-error | hung | isolation | all
//     --post-fix  additionally enforces the FIX-DEPENDENT assertions (the
//                 post-fix arms in README's row->scenario map). Default OFF:
//                 pre-fix runs must PASS on the
//                 mechanism-reproduction assertions alone (pre-fix posture), and
//                 a skipped fix-dependent check is reported SKIP, never PASS.
//
// Evidence sources:
//   - compose/oidc-dev/access.log — Caddy JSON access log (the wire truth). Field
//     realities are documented in the README ("Access-log field notes"): ts is an
//     EPOCH-SECONDS float (never Date.parse it), status is a top-level integer,
//     `duration` is seconds, and Caddy 2.11 REDACTS the values of Cookie AND
//     Authorization (presence is visible, values are not — the browser-side fetch
//     shim below supplies value-level evidence).
//   - Browser console shim (installed via addInitScript BEFORE any navigation):
//     TOKENS rows on localStorage["mogh-auth-tokens-v1"] transitions, LOADER rows
//     on LoadingScreen visibility transitions, FETCH rows (auth-header fingerprint)
//     for page-initiated requests, plus the app's own ws `on_login` console line.
//     Console rows cross mogh_ui's post-success `location.replace` reload because
//     they are captured at the CDP level, unlike any window-scoped state.
//
// Every scenario writes compose/oidc-dev/out/<scenario>.ndjson (request rows +
// console-shim rows) and prints per-assertion PASS/FAIL lines, then
// `SCENARIO <name> PASS|FAIL`. Exit code matches the verdict. Checks FAIL CLOSED:
// a check that cannot evaluate its evidence throws (rendered as FAIL), it never
// defaults to true.

import { chromium } from "playwright";
import { spawnSync } from "node:child_process";
import {
  mkdirSync,
  openSync,
  readSync,
  statSync,
  writeFileSync,
  closeSync,
} from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const LOG_PATH = path.join(HERE, "access.log");
const OUT_DIR = path.join(HERE, "out");
const COMPOSE_FILE = path.join(HERE, "..", "oidc-dev.compose.yaml");
const ENV_FILE = path.join(HERE, ".env");
const KOMODO_ORIGIN = "https://komodo.oidctest.localhost:8443";
const KOMODO_HOSTNAME = "komodo.oidctest.localhost"; // log rows carry :8443 too
const INTERACTION_RE = /\/interaction\//;

const DOCKER_HOST =
  process.env.DOCKER_HOST ?? "unix:///run/user/1000/podman/podman.sock";

// CLI -----------------------------------------------------------------------
const argv = process.argv.slice(2);
const POST_FIX = argv.includes("--post-fix");
const SCEN = argv.find((a) => !a.startsWith("--")) ?? "success";

// ---------------------------------------------------------------------------
// Evidence binding (round-8 F-001): a run's PASS lines certify a code tree,
// so the tree must be PINNED, not implied. Every run stamps HEAD into its
// scenario banner + ndjson meta rows, and refuses to run with uncommitted
// changes under the trees the suite asserts on (ui/src — the fix;
// compose/oidc-dev — the harness itself), because a dirty tree makes the
// stamped SHA a lie about what ran. BASELINE.md batch-2 certified a head two
// fix-commits stale exactly because nothing enforced this. Escape hatch for
// dev iterations: VERIFY_ALLOW_DIRTY=1 keeps the run going but stamps the
// dirt count into the meta rows — evidence can be recorded dirty, never
// mis-recorded clean.
// ---------------------------------------------------------------------------
const REPO_ROOT = path.join(HERE, "..", "..");
const GIT_HEAD = (() => {
  const r = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: REPO_ROOT,
    encoding: "utf8",
  });
  if (r.status !== 0) {
    throw new Error(
      `evidence binding: git rev-parse HEAD failed: ${r.stderr?.trim()}`,
    );
  }
  return r.stdout.trim();
})();
const DIRTY_COUNT = (() => {
  // Pathspec = the MERGE SURFACE the run actually exercises, not just the
  // fix's source dir (round-9 C-002): `ui` (the bundle's whole input tree —
  // package.json/yarn.lock/vite config feed core-ui.Dockerfile stage 1, not
  // only ui/src), `client/core/ts` (the yarn-linked client package the same
  // stage builds), `compose` (the stack topology: oidc-dev.compose.yaml sits
  // OUTSIDE compose/oidc-dev/), and .dockerignore (it decides what enters
  // the build context). node_modules/dist are gitignored and never trip
  // this. Acknowledged residual: the digest-pinned core image in
  // gitignored .env is outside any SHA — README documents it as a manual
  // step.
  const r = spawnSync(
    "git",
    [
      "status",
      "--porcelain",
      "--",
      "ui",
      "client/core/ts",
      "compose",
      ".dockerignore",
    ],
    { cwd: REPO_ROOT, encoding: "utf8" },
  );
  if (r.status !== 0) {
    throw new Error(
      `evidence binding: git status failed: ${r.stderr?.trim()}`,
    );
  }
  const dirty = r.stdout.split("\n").filter((l) => l.trim());
  if (dirty.length > 0 && !process.env.VERIFY_ALLOW_DIRTY) {
    throw new Error(
      `evidence binding (F-001): refusing to run — the merge-surface trees (ui, client/core/ts, compose, .dockerignore) are dirty, so a stamped SHA would not certify what actually ran:\n` +
        dirty.map((l) => `  ${l}`).join("\n") +
        `\ncommit first (the run protocol in BASELINE.md is: commit -> build -> run -> record). ` +
        `Dev iterations may set VERIFY_ALLOW_DIRTY=1 to proceed with the dirt stamped into the evidence.`,
    );
  }
  return dirty.length;
})();
console.log(
  `evidence: HEAD ${GIT_HEAD.slice(0, 12)}, asserted trees ${DIRTY_COUNT ? `DIRTY x${DIRTY_COUNT} (stamped)` : "clean"}`,
);

// ---------------------------------------------------------------------------
// Access-log tailer. Baselines the offset AT PROCESS START (the file persists
// across runs — a fresh process must never re-read prior runs' rows). Advances
// only to the last complete line so a torn write is re-read next poll instead
// of being silently dropped; handles truncation/rotation by reset on shrink.
// ---------------------------------------------------------------------------
const tailer = {
  offset: 0,
  entries: /** @type {any[]} */ ([]),
  timer: null,
  start() {
    try {
      // re-baseline AT START (the file persists across runs — a fresh process
      // must never re-read prior runs' rows)
      this.offset = statSync(LOG_PATH).size;
    } catch {
      throw new Error(
        `${LOG_PATH} not found — is the harness up? (touch compose/oidc-dev/access.log + up, see README "Run")`,
      );
    }
    this.entries = [];
    this.timer = setInterval(() => this.poll(), 250);
  },
  stop() {
    if (this.timer) clearInterval(this.timer);
    this.timer = null;
    this.poll(); // final drain
  },
  poll() {
    let size;
    try {
      size = statSync(LOG_PATH).size;
    } catch {
      return; // momentarily unlinked — retry next tick
    }
    if (size < this.offset) this.offset = 0; // truncated/rotated mid-run
    if (size === this.offset) return;
    let buf;
    try {
      const fd = openSync(LOG_PATH, "r");
      try {
        buf = Buffer.alloc(size - this.offset);
        readSync(fd, buf, 0, buf.length, this.offset);
      } finally {
        closeSync(fd);
      }
    } catch {
      return; // torn read — next poll retries
    }
    const text = buf.toString("utf8");
    const lastNewline = text.lastIndexOf("\n");
    if (lastNewline === -1) return; // no complete line yet — do not advance
    this.offset += lastNewline + 1;
    for (const line of text.slice(0, lastNewline).split("\n")) {
      if (!line.trim()) continue;
      try {
        this.entries.push(JSON.parse(line));
      } catch {
        // a malformed line is skipped, never fatal — but never a silent PASS
        // either: assertions throw when the rows they need are missing.
      }
    }
  },
};

// ---------------------------------------------------------------------------
// Log-row helpers. All time math stays in the log's own unit (epoch SECONDS);
// only cross-source comparisons (console rows are Date.now() ms) go via *1000.
// ---------------------------------------------------------------------------
const reqUri = (e) => String(e?.request?.uri ?? "");
const reqHost = (e) => String(e?.request?.host ?? "");
const isKomodoRow = (e) => reqHost(e).split(":")[0] === KOMODO_HOSTNAME;

// App-originated API surface (the UI's fetch + ws traffic). Document navigations
// and static assets are excluded — they are gate/browser artifacts, not the
// app-originated requests this suite bounds.
const API_RE = /^\/(user|read|execute|write|ws|auth\/login)(\/|$)/;
const isApi = (e) => isKomodoRow(e) && API_RE.test(reqUri(e));

const headerVal = (e, name, which = "request") => {
  // Field reality (observed during the suite's build): requests nest under
  // `request.headers`, but responses use `resp_headers` DIRECTLY as the header
  // map (no extra level).
  const headers = which === "request" ? e?.request?.headers ?? {} : e?.resp_headers ?? {};
  for (const k of Object.keys(headers)) {
    if (k.toLowerCase() === name) {
      const v = headers[k];
      return Array.isArray(v) ? String(v[0]) : String(v);
    }
  }
  return undefined;
};
const respHeaderVal = (e, name) => headerVal(e, name, "resp");
// Caddy redacts the VALUE ("[REDACTED]") — presence is still detectable.
const authPresent = (e) => {
  const v = headerVal(e, "authorization");
  return v !== undefined && v.length > 0;
};
const statusOf = (e) => {
  const s = e.status ?? e.resp_headers?.status ?? e.resp_headers?.Status;
  if (s === undefined) {
    throw new Error(
      `log row without a status field (uri=${reqUri(e)}) — Caddy field drift; ` +
        `check README "Access-log field notes"`,
    );
  }
  const n = typeof s === "number" ? s : parseInt(String(s).split(" ")[0], 10);
  if (!Number.isFinite(n)) throw new Error(`non-numeric status ${s}`);
  return n;
};
// Epoch-SECONDS float; never Date.parse. Throws on drift (fail closed).
const logSec = (e) => {
  const v = typeof e.ts === "number" ? e.ts : Number(e.ts);
  if (!Number.isFinite(v)) {
    throw new Error(
      `non-finite log timestamp ${e.ts} — Caddy ts is an epoch-seconds float; ` +
        `check README "Access-log field notes"`,
    );
  }
  return v;
};
const logMs = (e) => logSec(e) * 1000;
const exchangeRows = (log) =>
  log.filter((e) => reqUri(e).includes("/auth/login/ExchangeForJwt"));

// Bad-row disjunct — ONE count unit everywhere (amended once 500s were
// observed; see BASELINE.md "Rejection code is 500, not 401"):
// "unauthenticated OR 401/403 OR an auth-bearing 5xx". The 5xx term is not
// pedantry: on this core a bad-signature (escaped stale) token manifests as an
// AUTH-BEARING 500 on /user + /read, which a 401/403-only unit would read as
// zero. Shared by the success-window bound (windowBoundChecks) and the
// zero-residual window (residualRows). The /auth/login/* header-less
// exclusion stays FIRST: auth-surface discovery (GetLoginOptions, the
// header-less ExchangeForJwt POST) is exempt from both disjuncts by design.
const unauthOrFail = (e) => {
  const auth = authPresent(e);
  if (!auth && reqUri(e).startsWith("/auth/login/")) return false;
  const st = statusOf(e);
  // 429 is the per-IP limiter's own rejection (upstream draft (d) — the
  // endpoint of the "re-fired exchanges burn the budget into 429s"
  // narrative this suite exists to stop): a 429-only tail must count as
  // bad rows, not pass every disjunct silently (round-9 C-005).
  return (
    !auth || st === 401 || st === 403 || st === 429 || (st >= 500 && auth)
  );
};

// ---------------------------------------------------------------------------
// Browser instrumentation. Installed BEFORE any navigation so the exchange
// document's token write is captured. Console rows survive mogh_ui's
// location.replace reload (captured at CDP level) — window-scoped state does
// not, which is exactly why the shim reports over console.debug.
// ---------------------------------------------------------------------------
const TOKENS_KEY = "mogh-auth-tokens-v1";

// Structurally valid, EXPIRED stale jwt: header.payload.sub="stale-user".sig.
// mogh_auth_client attaches it blindly (jwt() reads memory only); core rejects
// the bad signature (500 — see the m1-seeded notes) — the M1 signal.
const STALE_JWT = [
  "eyJhbGciOiJIUzI1NiJ9",
  Buffer.from(JSON.stringify({ sub: "stale-user", exp: 1 })).toString(
    "base64url",
  ),
  "stale-signature",
].join(".");
// Header segments are identical across HS256 jwts — fingerprint by tail.
const jwtTail = (jwt) => String(jwt).slice(-24);
const STALE_TAIL = jwtTail(STALE_JWT);

// Structurally valid, sub-LESS jwt for the M2 forced drop: jwtDecode(jwt).sub
// is falsy -> add_and_change silently returns (tokens.js) while the exchange
// itself 200s — the exact silent-drop shape the m2-forced row asserts.
const SUBLESS_JWT = [
  "eyJhbGciOiJIUzI1NiJ9",
  Buffer.from(JSON.stringify({ exp: 9999999999 })).toString("base64url"),
  "sig",
].join(".");

const SHIM_SOURCE = /* js */ `
  (() => {
    const KEY = ${JSON.stringify(TOKENS_KEY)};
    const report = (tag, parts) => {
      try { console.debug(tag, Date.now(), ...parts); } catch {}
    };
    // initial store state (runs before any page script on every document)
    report("TOKENS", [String(localStorage.getItem(KEY))]);
    const origSet = Storage.prototype.setItem;
    localStorage.setItem = function (k, v) {
      try { if (k === KEY) report("TOKENS", [String(v)]); } catch {}
      return origSet.call(localStorage, k, v);
    };
    const origRemove = Storage.prototype.removeItem;
    localStorage.removeItem = function (k) {
      try { if (k === KEY) report("TOKENS", ["null"]); } catch {}
      return origRemove.call(localStorage, k);
    };
    // LoadingScreen visibility transitions (mogh_ui <Center><Loader/></Center>;
    // Mantine v9 keeps the semantic mantine-Loader-root class alongside the
    // m_-hash). Transition-only: no per-tick console noise.
    let on = false;
    setInterval(() => {
      const nowOn = !!document.querySelector(".mantine-Loader-root, .m_5ae2e3c");
      if (nowOn !== on) { on = nowOn; report("LOADER", [on ? "on" : "off"]); }
    }, 100);
    // fetch shim: auth-header fingerprint per page-initiated request. Caddy
    // redacts the authorization VALUE in the access log, so value-level
    // evidence (which token was attached) can only come from the page itself.
    const origFetch = window.fetch;
    window.fetch = function (input, init) {
      try {
        const url = typeof input === "string" ? input : (input && input.url) || "";
        let headers = (init && init.headers) || (input && input.headers);
        let auth;
        if (headers) {
          if (typeof headers.get === "function") auth = headers.get("authorization");
          else if (Array.isArray(headers)) {
            const hit = headers.find(([k]) => String(k).toLowerCase() === "authorization");
            auth = hit && hit[1];
          } else auth = headers["authorization"] ?? headers["Authorization"];
        }
        if (auth) {
          const u = new URL(url, location.origin);
          report("FETCH", [u.pathname + u.search, String(auth).slice(-24)]);
        }
      } catch {}
      return origFetch.apply(this, arguments);
    };
  })();
`;

async function attachShim(context) {
  await context.addInitScript(SHIM_SOURCE);
}

async function seedStaleToken(context) {
  // addInitScript runs BEFORE any page script (so mogh_auth_client's
  // module-load IIFE reads the seeded store — jwt() has no re-sync) — but
  // it ALSO runs on EVERY document. Guard on an existing store: seed only the
  // first document, otherwise the post-exchange reload would re-poison the
  // store with the stale token and the post-exchange reads would 500 again
  // (observed: the dashboard's own /user + GetCoreInfo came back stale-token).
  await context.addInitScript(
    `
    if (localStorage.getItem("mogh-auth-tokens-v1") === null) {
      localStorage.setItem("mogh-auth-tokens-v1", JSON.stringify({
        current: "stale-user",
        tokens: [{ user_id: "stale-user", jwt: ${JSON.stringify(STALE_JWT)} }],
      }));
    }
  `,
  );
}

// Console-row collector. Attached to the PAGE (not context) — Playwright has no
// context-level console event; every page of the context is driven explicitly.
function attachConsole(page, rows) {
  page.on("console", (msg) => {
    let text;
    try {
      text = msg.text();
    } catch {
      return;
    }
    const ts_ms = Date.now();
    if (text.startsWith("TOKENS")) {
      rows.push({ kind: "tokens", ts_ms, detail: text });
    } else if (text.startsWith("LOADER")) {
      rows.push({ kind: "loader", ts_ms, state: text.split(" ").pop(), detail: text });
    } else if (text.startsWith("FETCH")) {
      // "FETCH <ts> <path> <tail24>"
      const [, ts, p, tail] = text.split(" ");
      rows.push({ kind: "fetch", ts_ms: Number(ts), path: p, auth_tail: tail });
    } else if (/Logged into Update websocket/.test(text)) {
      rows.push({ kind: "ws-login", ts_ms, detail: text });
    }
  });
}

// ---------------------------------------------------------------------------
// The drive: portal session (phase A) then the real komodo OIDC login (phase B).
// The pinned mogh_ui LoginPage renders NO OIDC button, so the flow starts by
// direct navigation to core's /auth/oidc/login — identical to what
// mogh_auth_client's externalLogin builds. The whole komodo vhost is gated by
// the forward_auth, so the FIRST entry 302s to the portal's throwaway authorize
// (registered redirect_uri $PORTAL/dev); after that login the portal session
// exists and the SECOND entry auto-resumes through callback -> /?redeem_ready=true.
// ---------------------------------------------------------------------------
async function drive(page, { timeoutMs = 30_000 } = {}) {
  await page.goto(`${KOMODO_ORIGIN}/auth/oidc/login?redirect=%2F`, {
    timeout: timeoutMs,
    waitUntil: "domcontentloaded",
  });
  await page.waitForURL(INTERACTION_RE, { timeout: timeoutMs });
  // oidc-provider 9.12.2 dev-interaction form (verified from lib/views/login.js):
  // inputs name=login/password, submit button "Sign-in".
  await page.fill('input[name="login"]', "alice");
  await page.fill('input[name="password"]', "x");
  await page.click("button.login-submit");
  await page.waitForURL(/\/dev/, { timeout: timeoutMs }); // portal session established

  await page.goto(`${KOMODO_ORIGIN}/auth/oidc/login?redirect=%2F`, {
    timeout: timeoutMs,
    waitUntil: "domcontentloaded",
  });
  await page.waitForURL(/^https:\/\/komodo\.oidctest\.localhost/, {
    timeout: timeoutMs,
  }); // lands on /?redeem_ready=true, then sanitize-reloads to /
}

// ---------------------------------------------------------------------------
// Delay knob. The scenario recreates the delay sidecar itself (a DELAY_AUTH_MS
// change requires `up -d delay`), and restores 0 in a finally so a delayed run
// cannot poison the next scenario.
// ---------------------------------------------------------------------------
function compose(args, env = {}) {
  return spawnSync("docker-compose", ["-f", COMPOSE_FILE, "--env-file", ENV_FILE, ...args], {
    encoding: "utf8",
    env: { ...process.env, DOCKER_HOST, ...env },
  });
}
const spawnErr = (r) =>
  r.error ? ` (${r.error.message ?? r.error})` : ""; // SIG failures set only `error`
function getDelayMs() {
  const r = compose(["exec", "-T", "delay", "printenv", "DELAY_AUTH_MS"]);
  if (r.status !== 0) {
    throw new Error(
      `cannot read live DELAY_AUTH_MS from the delay container: ${r.stderr?.slice(0, 200)}${spawnErr(r)}`,
    );
  }
  return parseInt(String(r.stdout).trim(), 10) || 0;
}
async function setDelayMs(ms) {
  if (getDelayMs() === ms) return;
  const r = compose(["up", "-d", "delay"], { DELAY_AUTH_MS: String(ms) });
  if (r.status !== 0) {
    throw new Error(`up -d delay failed: ${r.stderr?.slice(0, 300)}${spawnErr(r)}`);
  }
  const deadline = Date.now() + 60_000;
  while (Date.now() < deadline) {
    try {
      if (getDelayMs() === ms) return;
    } catch {
      /* container restarting */
    }
    await sleep(500);
  }
  throw new Error(`delay knob did not reach ${ms}ms within 60s`);
}
// Crash-safety: a SIGINT mid-`hung` (or any failed restore) leaves
// DELAY_AUTH_MS set, and a later delayMs:null scenario would silently drive
// with a leftover knob. Every scenario therefore normalizes its EXPECTED knob
// (def.delayMs ?? 0) at start — loudly when a non-zero leftover was found —
// and the suite's resting state is 0.
async function normalizeKnob(expectedMs) {
  const live = getDelayMs();
  if (live !== expectedMs) {
    if (live !== 0) {
      console.error(
        `WARNING: leftover delay knob DELAY_AUTH_MS=${live} (expected ${expectedMs}) — resetting before this scenario`,
      );
    }
    await setDelayMs(expectedMs);
  }
}
// Signal path must be synchronous (spawnSync) so the recreate completes
// before the process dies.
function restoreKnobSync() {
  compose(["up", "-d", "delay"], { DELAY_AUTH_MS: "0" });
}
let signalSeen = false;
for (const sig of ["SIGINT", "SIGTERM"]) {
  process.on(sig, () => {
    if (signalSeen) return;
    signalSeen = true;
    console.error(`\n${sig} received — restoring delay knob to 0 before exit`);
    restoreKnobSync();
    process.exit(sig === "SIGINT" ? 130 : 143);
  });
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function waitForLog(pred, timeoutMs, desc) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const hit = tailer.entries.find(pred);
    if (hit) return hit;
    await sleep(150);
  }
  throw new Error(`timed out (${timeoutMs}ms) waiting for log row: ${desc}`);
}

// ---------------------------------------------------------------------------
// Assertion engine. FAIL CLOSED: a throwing check is a FAIL. Fix-dependent
// checks are SKIPped (not PASSed) when --post-fix is absent.
// ---------------------------------------------------------------------------
async function reportChecks(checks) {
  let pass = true;
  for (const c of checks) {
    if (c.fixDependent && !POST_FIX) {
      console.log(`SKIP (fix-dependent; rerun with --post-fix) ${c.name}`);
      continue;
    }
    if (c.preFixOnly && POST_FIX) {
      console.log(`SKIP (pre-fix mechanism assertion; not applicable with --post-fix) ${c.name}`);
      continue;
    }
    let ok = false;
    try {
      // Strict-true: a check must return TRUE to pass. `!== false` was
      // fail-open for `undefined` (review: the eternal-spinner check returned
      // undefined when zero LOADER rows were captured and PASSed). Every check
      // fn in this file returns a boolean or throws.
      ok = (await c.fn()) === true;
    } catch (e) {
      console.error(`    ↳ error: ${e.message ?? e}`);
      ok = false;
    }
    console.log(`${ok ? "PASS" : "FAIL"} ${c.name}`);
    pass = pass && ok;
  }
  return pass;
}
const check = (name, fn, opts = {}) => ({ name, fn, ...opts });

// ---------------------------------------------------------------------------
// Shared success-row window predicates. Window: [callback 303 -> dashboard
// rendered], where dashboard rendered = the exchange 200 happened and a
// subsequent authenticated GET /user returned 200 (a cheaper concrete
// stand-in for the DOM marker; final-URL post-sanitize is asserted alongside).
// ---------------------------------------------------------------------------
function anchorRows(log) {
  const callback = log.find(
    (e) =>
      reqUri(e).startsWith("/auth/oidc/callback") &&
      String(respHeaderVal(e, "location") ?? "").includes("redeem_ready=true"),
  );
  if (!callback) {
    throw new Error("no /auth/oidc/callback 303 -> /?redeem_ready=true row in window");
  }
  const exchange = exchangeRows(log).find((e) => statusOf(e) === 200);
  if (!exchange) throw new Error("no ExchangeForJwt 200 row");
  const dash = log.find(
    (e) =>
      reqUri(e) === "/user" &&
      statusOf(e) === 200 &&
      authPresent(e) &&
      logSec(e) >= logSec(exchange),
  );
  if (!dash) throw new Error("no authenticated GET /user 200 after the exchange");
  return { callback, exchange, dash };
}

function windowBoundChecks(log) {
  const { callback, dash } = anchorRows(log);
  const rows = log
    .filter((e) => isApi(e) && logSec(e) >= logSec(callback) && logSec(e) <= logSec(dash))
    .sort((a, b) => logSec(a) - logSec(b));
  const bad = rows.filter(unauthOrFail);
  // Sliding max ≤ 4 per any 15 s (the limiter's default window — a 5th failed
  // attempt trips it): for each row count rows in [t, t+15].
  let slideMax = 0;
  for (let i = 0; i < bad.length; i++) {
    let n = 0;
    for (let j = i; j < bad.length && logSec(bad[j]) - logSec(bad[i]) <= 15; j++) n++;
    slideMax = Math.max(slideMax, n);
  }
  return {
    rows,
    bad,
    checks: [
      check(
        `success-row unauth-or-401/403-or-auth-5xx window total <= 4 (observed ${bad.length})`,
        () => bad.length <= 4,
      ),
      check(
        `success-row unauth-or-401/403-or-auth-5xx sliding 15s max <= 4 (observed ${slideMax})`,
        () => slideMax <= 4,
      ),
    ],
  };
}

function successChecks(log, { rows, finalUrl }) {
  const { exchange, dash } = anchorRows(log);
  const checks = [
    // anchorRows() threw if any anchor was missing, so this asserts on the row
    // itself — never a bare `true`.
    check(
      `exchange 200 observed (ts=${exchange.ts.toFixed(3)})`,
      () => statusOf(exchange) === 200,
    ),
    check(
      ">=1 authenticated app request within 3.0s of exchange 200 (log-unit seconds)",
      () =>
        log.some(
          (e) =>
            isApi(e) &&
            authPresent(e) &&
            statusOf(e) === 200 &&
            logSec(e) - logSec(exchange) >= 0 &&
            logSec(e) - logSec(exchange) <= 3.0,
        ),
    ),
  ];
  checks.push(...windowBoundChecks(log).checks);
  // Websocket-alive: TWO signals (C-019, no default-to-success): an explicit
  // 101 row AND the app's own ws on_login console line within 5s of dashboard.
  const wsRow = log.find((e) => reqUri(e) === "/ws/update" && statusOf(e) === 101);
  checks.push(
    check(
      "update websocket upgrade row with explicit status 101",
      () => {
        if (!wsRow) {
          throw new Error(
            "no /ws/update row in the log — the 101 row is written when the connection CLOSES; the scenario closes the context to flush it",
          );
        }
        return true;
      },
    ),
    check(
      "update websocket browser-side liveness (on_login) within 5s of dashboard",
      () => {
        const hit = rows.find(
          (r) => r.kind === "ws-login" && r.ts_ms - logMs(dash) <= 5_000,
        );
        return !!hit;
      },
    ),
  );
  checks.push(
    check(
      "dashboard rendered (post-sanitize URL / and authenticated /user 200 in-window)",
      () => {
        const u = new URL(finalUrl);
        return u.pathname === "/" && !u.searchParams.has("redeem_ready") && !!dash;
      },
    ),
  );
  return checks;
}

// Flush the ws 101 row: Caddy writes the upgrade row only when the connection
// CLOSES, so the scenario closes the browser context and waits for the row.
// Returns true when the 101 row was observed (the CALLER's check must assert
// on this result, not re-query `log`: the row's `ts` is the UPGRADE time, not
// the close time — the preserved-session ws in exchange-error upgrades during
// pending, ~44 ms before the 401 — so any "after settlement" timestamp
// predicate on it is wrong; see the round-9 batch-4 lesson below).
async function flushWsRow(context) {
  await context.close();
  try {
    await waitForLog((e) => reqUri(e) === "/ws/update" && statusOf(e) === 101, 10_000, "/ws/update 101 row");
    return true;
  } catch {
    return false; // surfaced as a FAIL by the caller's ws check — no silent pass
  }
}

// Zero-residual window (failure rows): zero unauth-or-401/403-or-auth-5xx app requests and
// zero FAILED ws rows (status != 101) in the 60s AFTER settlement — the
// settlement row itself is excluded (strict >): the replayed exchange 401 IS
// the settlement, and mogh_ui attaches the still-stored jwt to it, so counting
// it would fail the post-fix run on the correct implementation's own row.
// Round-9 C-006: the ws half is only meaningful when the scenario FLUSHES the
// connection at close (the /ws/update row is written at close) — success,
// latency, exchange-error, and hung all do; a healthy OPEN connection writes
// no row at all, so "failed ws rows: 0" without a flush is vacuous.
function residualRows(log, settlementSec, seconds = 60) {
  const rows = log.filter(
    (e) =>
      isApi(e) &&
      logSec(e) > settlementSec &&
      logSec(e) <= settlementSec + seconds,
  );
  return {
    count: rows.filter(unauthOrFail).length,
    wsFails: rows.filter((e) => reqUri(e) === "/ws/update" && statusOf(e) !== 101)
      .length,
    rows,
  };
}

// ---------------------------------------------------------------------------
// Scenarios. Each returns { checks } after its drive; the runner handles the
// knob, the browser, the ndjson, and the verdict.
// ---------------------------------------------------------------------------
const SCENARIOS = {};

SCENARIOS.success = { run: async ({ context, page, rows, log }) => {
  await waitForLog(
    (e) => reqUri(e).includes("ExchangeForJwt") && statusOf(e) === 200,
    30_000,
    "ExchangeForJwt 200",
  );
  await sleep(8_000); // settle: reload, dashboard reads, ws connect
  const finalUrl = page.url();
  await flushWsRow(context); // the 101 row is written at ws close
  return { checks: successChecks(log, { rows, finalUrl }) };
  },
};

SCENARIOS.latency = { delayMs: 2000, run: async ({ context, page, rows, log, delayMs }) => {
  const ex = await waitForLog(
    (e) => reqUri(e).includes("ExchangeForJwt") && statusOf(e) === 200,
    30_000,
    "ExchangeForJwt 200",
  );
  const observedMs = (ex.duration ?? -1) * 1000; // Caddy `duration` is SECONDS
  await sleep(8_000);
  const finalUrl = page.url();
  await flushWsRow(context);
  const checks = successChecks(log, { rows, finalUrl });
  checks.push(
    check(
      `observed exchange round-trip >= DELAY_AUTH_MS (${delayMs}ms; observed ${observedMs.toFixed(0)}ms)`,
      () => observedMs >= delayMs * 0.98,
    ),
  );
  return { checks };
  },
};

SCENARIOS["m1-seeded"] = { delayMs: 1500, run: async ({ rows, log }) => {
  // DELAY_AUTH_MS=1500 widens the redeem window so the seeded stale-token
  // queries deterministically fire DURING it (the knob exists to widen the
  // race instead of relying on lucky timing). Under any watchdog.
  const ex = await waitForLog(
    (e) => reqUri(e).includes("ExchangeForJwt") && statusOf(e) === 200,
    30_000,
    "ExchangeForJwt 200",
  );
  const exSec = logSec(ex);
  // Wire-level M1 signal: /user or /read/* rows during the pending window that
  // carry an authorization header (Caddy shows presence, value REDACTED) and
  // are REJECTED, between drive start (first komodo row in this run) and the
  // exchange. REALITY (deviation from the original "server 401s it" assumption):
  // core answers a structurally-valid but bad-signature jwt with **500** on
  // /user and /read/GetCoreInfo (observed 2026-10-02) — the discriminating
  // signal is the stale-token-authenticated request inside the window, not the
  // exact rejection code, so 401/403/5xx all count.
  const driveStartSec = logSec(log[0] ?? ex) - 0.001;
  // (\/|$): `GET /user` has no trailing slash — `\/` alone silently dropped
  // it and the row was only ever matched via GetCoreInfo (review round 2).
  const staleRequest = (e) =>
    isApi(e) &&
    /^\/(user|read)(\/|$)/.test(reqUri(e)) &&
    authPresent(e) &&
    (statusOf(e) === 401 || statusOf(e) === 403 || statusOf(e) >= 500);
  const staleCandidates = log.filter(
    (e) =>
      staleRequest(e) && logSec(e) >= driveStartSec && logSec(e) <= exSec,
  );
  // Value-level corroboration (browser fetch shim — the log cannot show WHICH
  // token): a FETCH row carrying the stale jwt tail before the exchange row.
  // Scoped to the SAME paths the wire arm scopes (^/(user|read)): the redeem
  // request itself (/auth/login/ExchangeForJwt) legitimately carries the stale
  // jwt — it IS the credential being exchanged — and the shim logs that row at
  // DISPATCH, necessarily before the wire `ex` completion ts. Unscoped, the
  // exchange self-matches and false-fails the post-fix absence arm once the
  // real leaks are closed (pre-arm run: wire observed 0; sole stale-tail row
  // was the exchange dispatch at ex-1531 ms).
  const staleFetch = rows.find(
    (r) =>
      r.kind === "fetch" &&
      /^\/(user|read)(\/|$)/.test(r.path ?? "") &&
      r.auth_tail === STALE_TAIL &&
      r.ts_ms <= logMs(ex),
  );
  await sleep(8_000); // let the post-exchange reload + dashboard settle
  // Instrumentation liveness: an ABSENCE check (the post-fix arms) is only
  // meaningful if the shim provably captured something (review round 2 —
  // m2's shim-evidenced guard, applied to m1's negative arms too).
  const shimLive = () => {
    if (!rows.some((r) => r.kind === "fetch" || r.kind === "tokens")) {
      throw new Error(
        "instrumentation dead: zero FETCH/TOKENS rows captured — absence checks are meaningless",
      );
    }
  };
  const checks = [
    check(
      `exchange 200 observed (pending window closed; ts=${ex.ts.toFixed(3)})`,
      () => statusOf(ex) === 200,
    ),
    // Check names state the MODE-TRUE expectation (round-8 F-003): the
    // predicates invert under --post-fix, so a fixed name ("requests fire")
    // read as its own assertion inverted when pasted into issues. The name
    // now carries the expectation for the mode it runs in.
    check(
      `stale-token requests during the redeem window — expect ${POST_FIX ? "ZERO (fix defers provider reads)" : ">= 1 (pre-fix reproduction)"} (wire: auth+rejected in [drive start -> exchange 200]; observed ${staleCandidates.length}, statuses ${[...new Set(staleCandidates.map(statusOf))].join("/") || "-"})`,
      () => {
        if (POST_FIX) return staleCandidates.length === 0; // fix: provider-subtree reads deferred
        return staleCandidates.length >= 1;
      },
    ),
    check(
      `stale token VALUE on those paths — expect ${POST_FIX ? "ABSENT" : "PRESENT"} (fetch shim tail …${STALE_TAIL})`,
      () => {
        if (POST_FIX) {
          shimLive();
          return !staleFetch;
        }
        return staleFetch !== undefined;
      },
    ),
    check(
      "fresh token stored after the exchange (current pointer moved off stale-user)",
      () =>
        rows.some((r) => {
          if (r.kind !== "tokens" || r.ts_ms <= logMs(ex) || !r.detail) return false;
          try {
            // detail is the console text "TOKENS <ts> <store-json>"
            const store = JSON.parse(r.detail.slice(r.detail.indexOf("{")));
            return (
              store &&
              typeof store === "object" &&
              store.current != null &&
              store.current !== "stale-user"
            );
          } catch {
            return false; // unparsable row is not evidence of the transition
          }
        }),
    ),
  ];
  return { checks };
  },
};

// Route interception ONLY in this scenario (per plan), and it MUST be
// installed pre-drive: the exchange fires DURING the drive's second-entry
// cascade, so a route added after drive() never sees it (observed: the real
// jwt landed and the "drop" silently became a success row).
// ---------------------------------------------------------------------------
// m2 drive (shared): route-intercept the exchange and rewrite body.jwt to a
// structurally valid sub-less jwt. The wire still shows the true 200; the
// PAGE sees the dropped token. Run twice (round-9 C-003): session-less
// (`m2-forced`) and with a pre-existing session (`m2-seeded`) — the
// session-less shape is the only one the round-7 classifier handled; the
// seeded shape is the population its drift guard misread as key-drift.
// ---------------------------------------------------------------------------
const m2RouteHook = (context) =>
  context.route("**/auth/login/ExchangeForJwt", async (route) => {
    // Fetch the REAL upstream response, rewrite body.jwt to a structurally
    // valid sub-less jwt, fulfill. The wire still shows the true 200; the
    // PAGE sees the dropped token.
    const resp = await route.fetch();
    let body = {};
    try {
      body = await resp.json();
    } catch {}
    if (body && typeof body === "object") body.jwt = SUBLESS_JWT;
    await route.fulfill({ response: resp, json: body });
  });

const m2Checks = async ({ page, rows, log, seeded }) => {
  const ex = await waitForLog(
    (e) => reqUri(e).includes("ExchangeForJwt") && statusOf(e) === 200,
    30_000,
    "ExchangeForJwt 200 (upstream)",
  );
  await sleep(8_000); // reload lands on /login (pre-fix silent drop)
  const sublessTail = jwtTail(SUBLESS_JWT);
  const dropped = !rows.some(
    (r) => r.kind === "tokens" && r.detail && r.detail.includes(sublessTail),
  );
  const finalUrl = new URL(page.url());
  const checks = [
    check("token-store shim evidenced (>=1 TOKENS row captured)", () =>
      rows.some((r) => r.kind === "tokens"),
    ),
    check(
      `upstream exchange 200 observed (the drop is client-side; ts=${ex.ts.toFixed(3)})`,
      () => statusOf(ex) === 200,
    ),
    check(
      "exchanged (sub-less) jwt never lands in the token store (silent drop)",
      () => dropped,
    ),
    check(
      `silent drop path: reload lands on /login (observed ${finalUrl.pathname})`,
      () => finalUrl.pathname.startsWith("/login"),
    ),
    // FIX-DEPENDENT (the silent drop now surfaces): the drop flag surfaced on
    // the login page. STRICT PHASE (round-9 C-001): mere flag-presence
    // accepted a stale "ok" phase — an unconsumed "ok" on /login is exactly
    // the signature of a classifier that failed to verdict "drop" (the ok
    // branch consumes its flag BEFORE navigating), so presence-only made
    // this arm unable to detect its own failure mode. Phase must be "drop"
    // (or the toast text, when sessionStorage is degraded).
    check(
      "post-fix: drop flag surfaced in-document on the login page (phase drop)",
      async () => {
        const u = new URL(page.url());
        if (!u.pathname.startsWith("/login")) {
          throw new Error(`expected /login, at ${u.pathname}`);
        }
        const surfaced = await page.evaluate(() => {
          const flag = sessionStorage.getItem("komodo-redeem");
          if (flag) {
            try {
              if (JSON.parse(flag).phase === "drop") return "drop";
            } catch {
              // unparsable flag is not evidence of surfacing
            }
          }
          return /session could not be stored/i.test(document.body.innerText)
            ? "notification"
            : null;
        });
        if (!surfaced)
          throw new Error(
            "drop not surfaced: no phase-drop flag and no notification text (an ok-phase flag here means the classifier failed to verdict drop)",
          );
        return true;
      },
      { fixDependent: true },
    ),
  ];
  if (seeded) {
    // Round-9 C-003 coverage: the drop must verdict and surface even with a
    // pre-existing session, and the drop path must remain store-neutral —
    // the seeded session survives untouched (a drop writes nothing).
    checks.push(
      check(
        "post-fix: drop surfaced WITH a pre-existing session (round-9 C-003 population)",
        async () => {
          const u = new URL(page.url());
          if (!u.pathname.startsWith("/login")) {
            throw new Error(`expected /login, at ${u.pathname}`);
          }
          const surfaced = await page.evaluate(() => {
            const flag = sessionStorage.getItem("komodo-redeem");
            if (flag) {
              try {
                return JSON.parse(flag).phase === "drop";
              } catch {
                return false;
              }
            }
            return false;
          });
          if (!surfaced)
            throw new Error(
              "no phase-drop flag with a pre-existing session — the classifier misread the old closure token as drift (round-9 C-003 regression)",
            );
          return true;
        },
        { fixDependent: true },
      ),
      check(
        "pre-existing (seeded) session preserved through the silent drop (store neutrality)",
        () =>
          rows.every((r) => {
            if (r.kind !== "tokens" || !r.detail || !r.detail.includes("{"))
              return true;
            try {
              const store = JSON.parse(
                r.detail.slice(r.detail.indexOf("{")),
              );
              return (
                !store ||
                store.current == null ||
                store.current === "stale-user"
              );
            } catch {
              return true; // unparsable row is not evidence of a transition
            }
          }),
      ),
    );
  }
  return { checks };
};

SCENARIOS["m2-forced"] = {
  preDrive: m2RouteHook,
  run: async (ctx) => m2Checks({ ...ctx, seeded: false }),
};

SCENARIOS["m2-seeded"] = {
  preDrive: m2RouteHook,
  run: async (ctx) => m2Checks({ ...ctx, seeded: true }),
};

SCENARIOS["exchange-error"] = { run: async ({ context, page, rows, log }) => {
  // 1) a full successful login (settles on the dashboard), then 2) replay
  // /?redeem_ready=true: the pending login state is ONE-SHOT, so the fired
  // exchange 401s (and burns one of the 5 auth-limiter attempts).
  // Round-8 C-003: the leg-1 session DELIBERATELY survives the failed
  // settlement (failure paths are store-neutral), so the post-fix end state
  // is the APP rendered at "/" — RequireAuth sees the still-valid jwt — and
  // post-settlement traffic is authenticated 200s, which the zero-residual
  // predicate never counted. The old wipe-era expectation (final /login)
  // was an artifact of the store having been emptied; the arms below pin
  // the new contract: in-document convergence + session preservation.
  await waitForLog(
    (e) => reqUri(e).includes("ExchangeForJwt") && statusOf(e) === 200,
    30_000,
    "first ExchangeForJwt 200",
  );
  await sleep(5_000); // let the first login settle (reload -> dashboard)
  // Anchor for leg-2 (post-replay) observations: the eternal-spinner predicate
  // must only consider LOADER rows captured AFTER the goto — leg 1's redeem
  // spinner must not satisfy it (review round 2).
  const replayGotoMs = Date.now();
  await page.goto(`${KOMODO_ORIGIN}/?redeem_ready=true`, {
    timeout: 30_000,
    waitUntil: "domcontentloaded",
  });
  const fail = await waitForLog(
    (e) => reqUri(e).includes("ExchangeForJwt") && statusOf(e) === 401,
    30_000,
    "replayed ExchangeForJwt 401",
  );
  const settleSec = logSec(fail);
  await sleep(8_000); // N=8s no-navigation observation (pre-fix eternal spinner)
  const urlAtObserve = new URL(page.url());
  await sleep(60_000); // the zero-residual window itself — OBSERVE it (C-018)
  // Capture page state BEFORE flushWsRow closes the context (page.url() on a
  // closed page throws — round-9 C-006 lesson applied to this scenario too).
  const lastLoader = rows
    .filter((r) => r.kind === "loader" && r.ts_ms >= replayGotoMs)
    .at(-1);
  const finalUrl = new URL(page.url());
  // Flush the ws row BEFORE the verdict (round-9 C-006): the /ws/update row
  // is written when the connection CLOSES, so without a close, "failed ws
  // rows: 0" is vacuous — a healthy open connection writes no row at all.
  // Closing the context here flushes the preserved-session ws as 101
  // (asserted positively below); a genuinely FAILED upgrade writes its
  // non-101 row at failure time, inside the window, where residualRows
  // counts it.
  // The result — not a re-query of `log` — is the assertion: the flushed
  // row's ts is its UPGRADE time (during pending, ~44 ms before the 401),
  // so any timestamp predicate on it mis-fails. See round-9 batch-4 lesson.
  const wsFlushed101 = await flushWsRow(context);
  const residual = residualRows(log, settleSec, 60);
  // Round-8 C-003: the leg-1 session must SURVIVE the failed settlement.
  // Fail-closed against reintroducing the wipe: with the store emptied, no
  // authenticated request can occur post-settlement (hasJwt gates every
  // read), so zero auth'd /user rows = the wipe came back.
  const preserved = log.filter(
    (e) =>
      reqUri(e) === "/user" &&
      statusOf(e) === 200 &&
      authPresent(e) &&
      logSec(e) > settleSec,
  );
  // Document rows for the replayed /?redeem_ready=true navigation: post-fix the
  // settled-failed path converges IN-DOCUMENT (no reload), so there must be exactly
  // ONE (the goto itself); a reload would show a second.
  const replayDocRows = log.filter(
    (e) =>
      isKomodoRow(e) &&
      reqUri(e).startsWith("/?redeem_ready=true") &&
      String(e.request.headers?.["Sec-Fetch-Dest"] ?? "").includes("document") &&
      // the goto lands ~1-2s before the 401; keep the FIRST login's redeem
      // document row (≈10s earlier) out of this count
      logSec(e) >= settleSec - 5,
  );
  const checks = [
    check(
      `replayed exchange 401 observed (one-shot consumed session; ts=${fail.ts.toFixed(3)})`,
      () => statusOf(fail) === 401 && reqUri(fail).includes("ExchangeForJwt"),
    ),
    check(
      `pre-fix: no navigation for 8s after the 401 (URL still redeem_ready; observed ${urlAtObserve.pathname}${urlAtObserve.search})`,
      () =>
        urlAtObserve.searchParams.get("redeem_ready") === "true" &&
        urlAtObserve.pathname === "/",
      { preFixOnly: true },
    ),
    check(
      `pre-fix: eternal LoadingScreen (still up at end of observation; last state: ${lastLoader ? lastLoader.state : "none"})`,
      () => {
        if (!lastLoader) {
          throw new Error(
            "loader never observed after the replay goto — cannot distinguish eternal spinner from a dead instrument",
          );
        }
        return lastLoader.state === "on";
      },
      { preFixOnly: true },
    ),
    check(
      `post-fix: in-document convergence, session preserved (final ${finalUrl.pathname}; document loads for the replay: ${replayDocRows.length})`,
      () =>
        // no reload — the gate dropped in-document. The load COUNT is the
        // in-document signal; the pathname pins the store-neutral end state
        // (the surviving leg-1 session renders the app at "/", where the
        // wipe-era run bounced to /login with an emptied store).
        replayDocRows.length === 1 && finalUrl.pathname === "/",
      { fixDependent: true },
    ),
    check(
      `zero-residual 60s window (unauth-or-401/403-or-auth-5xx app rows: ${residual.count}; failed ws rows (flushed at close): ${residual.wsFails})`,
      () => residual.count === 0 && residual.wsFails === 0,
      { fixDependent: true },
    ),
    check(
      `post-fix: leg-1 session preserved through the failed settlement (authenticated /user 200 after ts=${settleSec.toFixed(3)}; observed ${preserved.length})`,
      () => preserved.length >= 1,
      { fixDependent: true },
    ),
    check(
      "post-fix: update websocket alive through the scenario (101 flushed at close)",
      () => {
        // Round-9 batch-4 lesson: assert the FLUSH RESULT, not a log scan —
        // the row's ts is its UPGRADE time (the preserved-session ws
        // upgrades during pending, ~44 ms before the 401), so "after
        // settlement" timestamp predicates mis-fail a healthy run.
        if (!wsFlushed101) {
          throw new Error(
            "flushWsRow timed out: no /ws/update 101 row within 10s of context close — no live ws through the scenario (round-9 C-006: without the flush this term was vacuous)",
          );
        }
        return true;
      },
      { fixDependent: true },
    ),
  ];
  console.log(
    `    ↳ observation (this phase): ${residual.count} unauth-or-401/403-or-auth-5xx rows, ${residual.wsFails} failed ws handshakes, ${preserved.length} authenticated /user 200 (leg-1 session preserved) in the 60s window`,
  );
  return { checks };
  },
};

SCENARIOS.hung = { delayMs: 15_000, run: async ({ context, page, rows, log, delayMs }) => {
  // DELAY_AUTH_MS=15000 — past the fix's 12s watchdog. PRE-FIX REALITY
  // (verified): there is no watchdog, so the LoadingScreen persists for the
  // FULL delay and the late 200 then stores the token and sanitize-reloads to
  // the dashboard (a slow success, NOT an eternal spinner — a finite delay
  // sidecar cannot black-hole the exchange forever; that arm stays with the
  // watchdog post-fix). Assertions therefore pin what pre-fix guarantees:
  // spinner up during the whole window, observed round-trip >= 15s, and the
  // post-callback observation window.
  const ex = await waitForLog(
    (e) => reqUri(e).includes("ExchangeForJwt") && statusOf(e) === 200,
    45_000,
    "late ExchangeForJwt 200 (>=15s)",
  );
  const observedMs = (ex.duration ?? -1) * 1000;
  const exMs = logMs(ex);
  // Spinner during [landing -> late 200]: a loader-ON event before the
  // exchange row and NO loader-OFF between that on and the exchange.
  const loaderOn = rows.filter((r) => r.kind === "loader" && r.state === "on" && r.ts_ms <= exMs);
  const gapOff = rows.find(
    (r) =>
      r.kind === "loader" &&
      r.state === "off" &&
      loaderOn.length > 0 &&
      r.ts_ms > loaderOn[0].ts_ms &&
      r.ts_ms < exMs,
  );
  await sleep(60_000); // post-settlement zero-residual window (observed)
  // Capture the landing BEFORE flushWsRow closes the context (round-9 C-006 —
  // page.url() on a closed page throws), then flush: the /ws/update row is
  // written at close, so without it "failed ws: 0" was vacuous.
  const finalUrlAtEnd = new URL(page.url());
  const wsFlushed101 = await flushWsRow(context);
  const residual = residualRows(log, logSec(ex), 60);
  const checks = [
    check(
      `observed exchange round-trip >= DELAY_AUTH_MS (${delayMs}ms; observed ${observedMs.toFixed(0)}ms)`,
      () => observedMs >= delayMs * 0.98,
    ),
    check(
      `pre-fix: LoadingScreen up with no OBSERVED gap during [redeem landing -> late 200] (${loaderOn.length ? (exMs - loaderOn[0].ts_ms).toFixed(0) : "no loader-on"}ms observed; gap-off=${!!gapOff}; 100ms poll can miss sub-100ms transitions)`,
      () => loaderOn.length >= 1 && !gapOff,
      { preFixOnly: true },
    ),
    check(
      `post-fix: watchdog dropped the gate BEFORE the late 200 (loader-off observed pre-settlement)`,
      () => loaderOn.length >= 1 && !!gapOff,
      { fixDependent: true },
    ),
    check(
      `post-fix: zero-residual 60s window after the watchdog settlement (unauth-or-401/403-or-auth-5xx: ${residual.count}; failed ws rows (flushed at close): ${residual.wsFails})`,
      () => residual.count === 0 && residual.wsFails === 0,
      { fixDependent: true },
    ),
    check(
      "post-fix: update websocket alive through the scenario (101 flushed at close)",
      () => {
        // Same flush-result contract as exchange-error: the row's ts is its
        // UPGRADE time, so log scans with "after X" predicates mis-fail.
        if (!wsFlushed101) {
          throw new Error(
            "flushWsRow timed out: no /ws/update 101 row within 10s of context close — no live ws through the scenario (round-9 C-006)",
          );
        }
        return true;
      },
      { fixDependent: true },
    ),
    check(
      `post-fix: watchdog converges (gate drops -> /login, then the late-success redirect recovers)`,
      async () => {
        const u = finalUrlAtEnd;
        // Watchdog fired at ~12s while the exchange was still in flight; the
        // late 200 (t≈15s) then stores the token and reloads — the one-shot
        // must send the user on to `/` (never left stranded mid-gate).
        if (u.pathname !== "/") {
          throw new Error(`expected final landing /, at ${u.pathname}${u.search}`);
        }
        return true;
      },
      { fixDependent: true },
    ),
  ];
  console.log(
    `    ↳ pre-fix observation: after the late 200 the URL is ${new URL(page.url()).pathname}${new URL(page.url()).search} (slow-success reload; the hung row's pre-fix cell in README's scenario map documents this)`,
  );
  return { checks };
  },
};

SCENARIOS.isolation = { run: async ({ page, rows, log }) => {
  await waitForLog(
    (e) => reqUri(e).includes("ExchangeForJwt") && statusOf(e) === 200,
    30_000,
    "ExchangeForJwt 200",
  );
  await sleep(5_000); // dashboard settled
  // Gate-isolation (C-009): a failing NON-redeem execute must not flip the
  // gate. Fire it exactly like the app's useWrite would (authorization +
  // JSON body) with a deployment id that cannot exist -> 4xx/5xx.
  const probe = await page.evaluate(async () => {
    const store = JSON.parse(localStorage.getItem("mogh-auth-tokens-v1") ?? "null");
    const jwt = store?.tokens?.find((t) => t.user_id === store.current)?.jwt ?? "";
    window.__isoMarker = Date.now();
    const r = await fetch("/execute/StartDeployment", {
      method: "POST",
      headers: { authorization: jwt, "content-type": "application/json" },
      body: JSON.stringify({ params: { deployment: "nonexistent-harness-probe" } }),
    });
    return { status: r.status };
  });
  await sleep(2_000); // the flip would happen within this window if ever
  const markerAlive = await page.evaluate(() => window.__isoMarker ?? null).catch(() => null);
  const url = new URL(page.url());
  const loaderFlip = rows.find(
    (r) => r.kind === "loader" && r.state === "on" && markerAlive && r.ts_ms > markerAlive,
  );
  const checks = [
    check(
      `probe failing execute returned non-auth 4xx/5xx (observed ${probe.status})`,
      () => probe.status >= 400 && probe.status !== 401 && probe.status !== 403,
    ),
    check(
      `no full-page reload to /login within 2s (URL ${url.pathname}; marker ${markerAlive ? "alive" : "LOST"})`,
      () => url.pathname === "/" && markerAlive !== null,
    ),
    check(`no LoadingScreen flip within 2s (flip observed: ${!!loaderFlip})`, () => !loaderFlip),
  ];
  return { checks };
  },
};

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------
async function runScenario(name) {
  const def = SCENARIOS[name];
  if (!def) {
    throw new Error(
      `unknown scenario "${name}" — valid: ${Object.keys(SCENARIOS).join(", ")}, all`,
    );
  }
  const delayMs = def.delayMs ?? null;
  console.log(
    `\n=== scenario ${name}${POST_FIX ? " (--post-fix)" : ""} (head ${GIT_HEAD.slice(0, 12)}${DIRTY_COUNT ? `, DIRTY x${DIRTY_COUNT}` : ""}) ===`,
  );
  const rows = [
    {
      kind: "meta",
      scenario: name,
      postFix: POST_FIX,
      delayMs,
      head: GIT_HEAD,
      dirty: DIRTY_COUNT,
      startedAt: new Date().toISOString(),
    },
  ];
  let browser;
  let pass = false;
  let knobOk = true; // a failed restore must FAIL the run, not just log
  try {
    // Normalize FIRST: a leftover knob from a crashed earlier run (SIGINT
    // mid-hung, failed restore) would silently poison THIS scenario's drive.
    await normalizeKnob(delayMs ?? 0);
    if (delayMs !== null) console.log(`delay knob active: DELAY_AUTH_MS=${delayMs}`);
    tailer.start();
    browser = await chromium.launch();
    const context = await browser.newContext({ ignoreHTTPSErrors: true });
    if (name === "m1-seeded" || name === "m2-seeded")
      await seedStaleToken(context); // BEFORE the shim so its initial report shows the seed
    await attachShim(context); // BEFORE any navigation
    const page = await context.newPage();
    attachConsole(page, rows);
    if (def.preDrive) await def.preDrive(context); // BEFORE drive: the exchange fires during it
    await drive(page);
    const { checks } = await def.run({ context, page, rows, log: tailer.entries, delayMs });
    pass = await reportChecks(checks);
  } catch (e) {
    console.error(`FAIL scenario driver error: ${e.message ?? e}`);
    pass = false;
  } finally {
    tailer.stop();
    for (const e of tailer.entries) {
      rows.push({
        kind: "req",
        ts: e.ts,
        ts_ms: Number.isFinite(Number(e.ts)) ? Math.round(Number(e.ts) * 1000) : null,
        method: e.request?.method,
        host: e.request?.host,
        url: e.request?.uri,
        auth: authPresent(e),
        status: (() => {
          try {
            return statusOf(e);
          } catch {
            return null;
          }
        })(),
        duration_s: e.duration,
      });
    }
    // Evidence write is BEST-EFFORT and must not jump the cleanup queue
    // (round-10 C-005): a throw here (EACCES/ENOSPC on out/) used to
    // propagate out of the finally and skip browser.close() AND the
    // delay-knob restore — leaking Chromium and leaving DELAY_AUTH_MS set,
    // the exact leftover the SIGINT handler and knobOk exist to prevent.
    // Cleanup order: flush evidence (guarded) -> close browser -> restore
    // the shared knob -> then compute the verdict.
    try {
      mkdirSync(OUT_DIR, { recursive: true });
      writeFileSync(
        path.join(OUT_DIR, `${name}.ndjson`),
        rows.map((r) => JSON.stringify(r)).join("\n") + "\n",
      );
      console.log(`ndjson: ${path.join(OUT_DIR, name)}.ndjson (${rows.length} rows)`);
    } catch (e) {
      console.error(`FAIL evidence write (continuing cleanup): ${e.message ?? e}`);
      pass = false; // a run whose evidence could not be recorded is not a PASS
    }
    if (browser) await browser.close().catch(() => {});
    // Restore unconditionally (every scenario's expected resting state is 0,
    // including the delayMs:null rows that normalized a leftover away).
    try {
      if (getDelayMs() !== 0) await setDelayMs(0);
    } catch (e) {
      console.error(`FAIL knob restore: ${e.message ?? e}`);
      knobOk = false;
    }
  }
  if (!knobOk) pass = false; // verdict computed after the finally block runs
  console.log(`SCENARIO ${name} ${pass ? "PASS" : "FAIL"}`);
  return pass;
}

const ALL = [
  "success",
  "latency",
  "m1-seeded",
  "m2-forced",
  "m2-seeded",
  "exchange-error",
  "hung",
  "isolation",
];

if (SCEN === "all") {
  let allPass = true;
  for (const name of ALL) {
    // 16s gap between scenarios: the auth rate limiter is 5 attempts / 15s
    // keyed by IP, and all scenarios share the Caddy egress IP — do not let
    // one row's burned attempts feed the next row's exchange.
    if (name !== ALL[0]) await sleep(16_000);
    allPass = (await runScenario(name)) && allPass;
  }
  console.log(`\nALL SCENARIOS ${allPass ? "PASS" : "FAIL"}`);
  // exitCode + natural exit (not process.exit): a hard exit can truncate
  // buffered stdout when the output is piped, and the baseline runs parse
  // these lines.
  process.exitCode = allPass ? 0 : 1;
} else {
  const pass = await runScenario(SCEN);
  process.exitCode = pass ? 0 : 1;
}
