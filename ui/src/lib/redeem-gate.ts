import { useSyncExternalStore } from "react";
import type { MutationCacheConfig, QueryClient } from "@tanstack/react-query";
import { MoghAuth } from "komodo_client";

// Redeem lifecycle. "idle" means OPEN everywhere: only a document
// that itself arms the redeem mutation can leave idle, so every gate keyed on
// `!== "pending"` behaves exactly like today on normal page loads.
// Delivery fact (react-query internals, verified against the pinned
// query-core 5.102.4): MutationCache listeners run SYNCHRONOUSLY inside the
// dispatch's task — mutationCache.notify invokes this.listeners.forEach
// directly inside notifyManager.batch, and batch runs its callback
// immediately; the systemSetTimeoutZero scheduler delivers only the
// notifyManager's observer-notification QUEUE (schedule/batchCalls), which
// cache listeners never touch. A render-phase dispatch notifies zero
// effect-scoped subscribers, deterministically. Arming uses the config
// hooks, which run inside execute regardless.
export type RedeemState = "idle" | "pending" | "settled-ok" | "settled-failed";

const REDEEM_KEY = "ExchangeForJwt";
// Browser fetch has no default timeout — without this bound a black-holing
// proxy would leave the gate "pending" forever.
const WATCHDOG_MS = 12_000;
// "Just over the watchdog": the late-success redirect accepts only a fresh
// flag, and the settlement-time rewrite restarts the flag TTL, so freshness
// measures from the last confirmed settlement, not the original arm.
const FLAG_TTL_MS = 15_000;
const FLAG_KEY = "komodo-redeem";
// Exported as the documented seam; every read of the key goes through
// readMoghStore() below — one parse, one source of truth for the mogh
// storage key. (Hosts must read storage directly: mogh_auth_client's IIFE
// snapshots localStorage once at module init and never re-syncs, so its
// closure view is not trustworthy mid-session.)
export const MOGH_TOKENS_KEY = "mogh-auth-tokens-v1"; // mogh_auth_client 1.7.1 tokens.js:5

let state: RedeemState = "idle";
let watchdog: ReturnType<typeof setTimeout> | undefined;
const listeners = new Set<() => void>();

// One stable subscribe for both hooks: hoisted to module scope so re-renders
// reuse it instead of re-creating an identical closure on every call.
const subscribe = (cb: () => void) => {
  listeners.add(cb);
  return () => {
    listeners.delete(cb);
  };
};

// mutationKey is `readonly unknown[]` (query-core MutationKey) — readonly is
// load-bearing: a mutable `unknown[]` here fails structural assignability.
const isExchange = (
  m: { options?: { mutationKey?: readonly unknown[] } } | undefined,
) => m?.options?.mutationKey?.[0] === REDEEM_KEY;

function setState(next: RedeemState) {
  if (state === next) return;
  // The flip is synchronous inside the dispatch callback, before any render
  // it could trigger: listeners fire and React re-renders with the settled
  // state in the same task.
  //
  // Store hygiene is deliberately failure-NEUTRAL. An earlier
  // revision ran LOGIN_TOKENS.remove_all() here; that emptied the
  // profile-wide store on EVERY failure path — watchdog on a black-holed
  // proxy, a replayed ?redeem_ready=true link, a spent one-shot session —
  // silently logging every tab out of a perfectly valid pre-existing session.
  // mogh's add_and_change is all-or-nothing (tokens.js: falsy sub
  // early-returns; there is no partial write), so a failed exchange leaves no
  // half-state to clean: any token in the store predates the redeem and
  // belongs to a session this flow must preserve. Reads that resume
  // post-settlement with that token are the app's ordinary session behavior
  // (valid -> 200s; expired -> 401 into mogh_ui's own handling) — and neither
  // is limiter traffic: the per-IP limiter counts /auth/login attempts, not
  // rejected app reads.
  state = next;
  if (state !== "pending" && watchdog !== undefined) {
    clearTimeout(watchdog);
    watchdog = undefined;
  }
  listeners.forEach((l) => l());
}

// Exception-free helper: mutation.cjs awaits config onSuccess BEFORE mogh_ui's
// token write with no per-hook guard (only the error-path hooks are wrapped) —
// a throw here converts a 200 exchange into the failure path (a silent drop).
function safe<T>(fn: () => T, label: string): T | undefined {
  try {
    return fn();
  } catch (e) {
    console.error(`redeem-gate: ${label} failed (ignored)`, e);
    return undefined;
  }
}

// The mogh token store, parsed exactly once per check (the key/schema live in
// mogh_auth_client ^1.7.1's tokens.js — ONE reader here, so a shape change
// upstream has one place to surface). Absence and corruption both yield
// undefined; on a parse failure a redacted fingerprint (length + 8-char head
// — NEVER the raw value, which can carry every stored jwt tail) is logged
// with the key named, so support can identify the corruption class without
// credentials riding in a pasted log.
type MoghTokenStore = {
  current?: string;
  tokens?: Array<{ user_id: string; jwt: string }>;
};

function readMoghStore(): MoghTokenStore | undefined {
  const raw = safe(
    () => localStorage.getItem(MOGH_TOKENS_KEY),
    "mogh token store read",
  );
  // Absence is NOT corruption — it is the m2 canonical silent-drop shape
  // (mogh simply never wrote the key). Return quietly: logging here would
  // cry wolf on every real drop, and `raw: null` would be indistinguishable
  // from literal "null" bytes. undefined (the read itself threw) was already
  // logged by safe()'s label above — quiet here too.
  if (raw === null || raw === undefined) return undefined;
  // safe()'s label deliberately carries NO raw value: labels print on
  // failure, and a failure here is exactly the moment raw bytes could leak
  // into a pasted support log. The redacted fingerprint below covers every
  // path that reaches `return undefined` — a THROWING parse lands here via
  // safe()'s undefined, and JSON.parse("null") lands here via the !parsed
  // check without ever throwing.
  const parsed = safe(() => JSON.parse(raw), "mogh token store parse");
  if (!parsed || typeof parsed !== "object") {
    console.error(
      `redeem-gate: mogh token store for "${MOGH_TOKENS_KEY}" is not a store (JSON null/non-object/unparsable; raw length ${raw.length}, head ${JSON.stringify(raw.slice(0, 8))}) — treating as corrupt`,
    );
    return undefined;
  }
  return parsed as MoghTokenStore;
}

/** Schema-shaped presence check: the store parsed and holds a token entry. */
export function hasStoredJwt(): boolean {
  const store = readMoghStore();
  return Array.isArray(store?.tokens) && store.tokens.length > 0;
}

// PRE-ARM: mogh_ui fires the redeem during ROUTER's
// render — a CHILD of WebsocketProvider, whose gated reads commit
// enabled=true before any child-render flip can exist (React renders
// parent-first). Initializing pending at module scope closes that gap:
// the provider's FIRST render already sees the gate closed. The watchdog
// arms here too, so the window stays bounded even if the real mutation
// never fires (defensive: mogh_ui version drift). Runs at import —
// main.tsx's import guarantees this is before any React render — and the
// later real onMutate flip is idempotent (equality guard; its watchdog
// arm is a no-op because watchdog is already set). Normal loads (no
// redeem_ready param) stay idle, byte-identical to pre-fix behavior.
if (
  typeof window !== "undefined" &&
  new URLSearchParams(window.location.search).get("redeem_ready") === "true"
) {
  state = "pending";
  watchdog = setTimeout(() => {
    watchdog = undefined;
    setState("settled-failed");
  }, WATCHDOG_MS);
}

const sessionStorageOk =
  typeof window !== "undefined" &&
  safe(() => {
    const k = `${FLAG_KEY}-probe`;
    window.sessionStorage.setItem(k, "1");
    window.sessionStorage.removeItem(k);
    return true;
  }, "sessionStorage probe") === true;

// Single evidence flag (the evidence-flag contract): never carries the raw jwt.
type RedeemFlag = { t: number; phase: "ok" | "drop" };

function writeFlag(phase: RedeemFlag["phase"]) {
  if (!sessionStorageOk) {
    // Degraded mode: this module IS the drop detector, so
    // the drop verdict must still surface — console-only signal, then skip.
    if (phase === "drop") {
      console.error(
        "redeem-gate: exchange 200'd but jwt not stored (silent drop); sessionStorage unavailable — console-only signal",
      );
    }
    return;
  }
  safe(
    () =>
      window.sessionStorage.setItem(
        FLAG_KEY,
        JSON.stringify({ t: Date.now(), phase }),
      ),
    "flag write",
  );
}

export function readRedeemFlag(): RedeemFlag | undefined {
  if (!sessionStorageOk) return undefined;
  return safe(() => {
    const raw = window.sessionStorage.getItem(FLAG_KEY);
    return raw ? (JSON.parse(raw) as RedeemFlag) : undefined;
  }, "flag read");
}

export function consumeRedeemFlag() {
  if (!sessionStorageOk) return;
  safe(() => window.sessionStorage.removeItem(FLAG_KEY), "flag clear");
}

export function flagFresh(flag: RedeemFlag | undefined): boolean {
  return !!flag && flag.phase === "ok" && Date.now() - flag.t < FLAG_TTL_MS;
}

/** True unless a redeem mutation is in flight. The one gate predicate. */
export function useRedeemGateOpen(): boolean {
  return useSyncExternalStore(
    subscribe,
    () => state !== "pending",
    () => true,
  );
}

export function useRedeemState(): RedeemState {
  return useSyncExternalStore(
    subscribe,
    () => state,
    () => "idle" as RedeemState,
  );
}

/** Cache-config hooks: synchronous inside execute, before mogh_ui's handlers. */
export function createRedeemGateHooks(): MutationCacheConfig {
  return {
    onMutate: (_variables, mutation) => {
      if (!isExchange(mutation)) return;
      setState("pending");
      safe(() => {
        if (watchdog === undefined) {
          watchdog = setTimeout(() => {
            watchdog = undefined;
            // Fail-safe convergence; a late success still recovers via
            // mogh_ui's own handler + the login-page redirect.
            setState("settled-failed");
          }, WATCHDOG_MS);
        }
      }, "watchdog arm");
    },
    onSuccess: (_data, _variables, _context, mutation) => {
      if (!isExchange(mutation)) return;
      // Pre-navigation evidence: exchange 200'd; mogh_ui's write+reload next.
      writeFlag("ok");
    },
  };
}

/** Settlement subscription; lives for the page's lifetime. */
export function initRedeemGate(client: QueryClient) {
  client.getMutationCache().subscribe((event) => {
    if (event.type !== "updated" || !isExchange(event.mutation)) return;
    // action is a REQUIRED discriminated union (ContinueAction | ErrorAction |
    // … | SuccessAction) — narrow it; the previous `{ type?: string } |
    // undefined` cast claimed action could be absent, which the .d.ts rules out.
    if (event.action.type === "success") {
      // Fires AFTER mogh_ui's onSuccess. Silent-drop check: did storage gain
      // the jwt? action.data is the exact success-dispatch payload (identical
      // to mutation.state.data per mutation.cjs's success reducer).
      const jwt = (event.action.data as { jwt?: string } | undefined)?.jwt;
      let phase: RedeemFlag["phase"] = "ok";
      if (jwt) {
        // Derive the shape FIRST, then use it. Optional chaining
        // short-circuits only on nullish: `store?.tokens?.some(...)` still
        // THREW for `{"tokens":"junk"}` (calling undefined) and for
        // `{"tokens":[null]}` (t.jwt inside the callback) — in the cache
        // subscription, skipping writeFlag + settled-ok. Array.isArray
        // derivation cannot throw; everything below works on `tokens`.
        const store = readMoghStore();
        const tokens = Array.isArray(store?.tokens) ? store.tokens : undefined;
        const jwtAbsent = !tokens?.some((t) => !!t && t.jwt === jwt);
        // Schema guard — a "drop" verdict needs the absence to be
        // TRUSTWORTHY. Four populations reach this line with jwtAbsent
        // true, discriminated by what the library's closure view holds (a
        // successful add_and_change sets that view to the EXCHANGED jwt
        // in-process; a dropped one leaves it untouched):
        //  1. store absent, closure EMPTY — the m2 canonical silent drop
        //     (add_and_change swallowed the token; nothing was written
        //     anywhere)                                        -> "drop"
        //  2. store holds a PRE-EXISTING session, closure = that OLD token
        //     (a truthiness check here would misread this population as
        //     drift — a dropped exchange in a logged-in document would go
        //     completely silent)                                  -> "drop"
        //  3. closure === THE EXCHANGED jwt while the pinned key lacks it —
        //     the library wrote SOMEWHERE ELSE (upstream key/schema rename
        //     under ^1.7.1); our absence reading is stale by construction
        //                                               -> drift, loud "ok"
        //  4. the pinned key parsed but is not the expected
        //     `{ tokens: [...] }` schema — a non-store wrote this key
        //                                               -> drift, loud "ok"
        // Population 3 must fail toward "ok" (never a false drop);
        // populations 1-2 are the real drops this detector exists for.
        // EQUALITY is the discriminator — truthiness would collapse
        // populations 2 and 3 into one wrong verdict.
        const libJwt = safe(() => MoghAuth.LOGIN_TOKENS.jwt(), "jwt read");
        const driftSuspected =
          (store !== undefined && tokens === undefined) ||
          (jwtAbsent && libJwt === jwt);
        if (driftSuspected) {
          console.error(
            `redeem-gate: mogh token store shape unrecognized for "${MOGH_TOKENS_KEY}" — silent-drop detection unavailable (library closure view holds the EXCHANGED jwt: upstream key/schema drift suspected)`,
          );
        } else if (jwtAbsent) {
          phase = "drop";
        }
      }
      writeFlag(phase); // refresh t: late-success TTL runs from here
      setState("settled-ok");
    } else if (event.action.type === "error") {
      setState("settled-failed");
    }
  });
}
