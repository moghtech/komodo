import { useEffect } from "react";
import { notifications } from "@mantine/notifications";
import { LoginPage } from "mogh_ui";
import { useUserInvalidate } from "@/lib/hooks";
import {
  consumeRedeemFlag,
  flagFresh,
  hasStoredJwt,
  readRedeemFlag,
} from "@/lib/redeem-gate";

// Origin guard for the late-success redirect target. backto
// survives mogh_ui's post-exchange sanitize (utils.js strips only
// redeem_ready|totp|passkey), so a crafted link —
// /login?backto=//evil.example&redeem_ready=true — would otherwise navigate
// off-origin immediately after the victim logs in. Accept ONLY a local path:
// "/" exactly, or "/" followed by a character that is neither "/"
// (protocol-relative //host) nor "\" (WHATWG URL parsing folds "\" to "/" for
// special schemes, so /\evil.example is protocol-relative too; the parser is
// the spec, not a browser quirk), and NO control character anywhere — the
// parser strips tabs/newlines BEFORE parsing, so "/\t/evil.example" would
// weld into "//evil.example" (protocol-relative) only AFTER this guard has
// approved it; control chars never belong in a local path. Anything else —
// absolute URLs, empty, null — falls back to "/". Query strings on an
// accepted path are fine: they stay on this origin, and a percent-encoded
// "%2F%2F" stays a literal on-origin path segment (URLSearchParams already
// decoded the one layer that mattered).
function safeLocalPath(raw: string | null): string {
  // typeof guard FIRST: `raw === "/"` being false does not exclude null, so
  // without it null flows into the regex call (tsc caught exactly that).
  return (
    typeof raw === "string" &&
    !/[\x00-\x1f\x7f]/.test(raw) &&
    (raw === "/" || /^\/[^/\\]/.test(raw))
  )
    ? raw
    : "/";
}

// Per-document latch for the silent-drop toast: with the drop flag no longer
// consumed on read, the effect re-runs on every /login mount within the
// flag's TTL (and StrictMode's dev double-mount re-runs it too); one toast
// per document, not one per mount.
let dropNoticeShown = false;

export default function Login(props: {
  passkeyIsPending?: boolean;
  totpIsPending?: boolean;
}) {
  const userInvalidate = useUserInvalidate();

  // One-shot late-success redirect + silent-drop surfacing.
  // With no flag this effect is a single sessionStorage read, so every
  // unflagged visit behaves exactly as before.
  //
  // Window APIs only, deliberately — NO useNavigate/useSearchParams:
  // router.tsx renders <Login> for the passkey/totp branch BEFORE
  // <BrowserRouter> mounts, where router-context hooks throw. Reading
  // location.search and redirecting via location.replace mirrors mogh_ui's
  // own maybeNavigate — but the target guard below deliberately DIVERGES:
  // mogh_ui's maybeNavigate replaces whatever backto carries, unvalidated,
  // and this PR must not add another unguarded call site of a flaw it
  // documents.
  useEffect(() => {
    // Sanitize backto IN THE URL at mount, before anything reads it. This
    // page renders mogh_ui's LoginPage, whose own success handler
    // (maybeNavigate) replaces whatever location.search's backto carries —
    // unvalidated — after a LOCAL/passkey/totp login. safeLocalPath guards
    // only this file's own redirect; the hostile value must be removed from
    // the URL itself so every consumer (mogh_ui's included) sees only a
    // local path. replaceState: no navigation, LoginPage unaffected.
    const rawBackto = new URLSearchParams(window.location.search).get("backto");
    // Compute the guard ONCE and write its answer: safeLocalPath is this
    // file's security seam, and two invocations are two chances for a future
    // edit to make "what was approved" and "what was written" diverge.
    const sanitizedBackto = rawBackto === null ? null : safeLocalPath(rawBackto);
    if (sanitizedBackto !== null && sanitizedBackto !== rawBackto) {
      const url = new URL(window.location.href);
      url.searchParams.set("backto", sanitizedBackto);
      window.history.replaceState(null, "", url.pathname + url.search);
    }
    const flag = readRedeemFlag();
    if (!flag) return;
    if (flag.phase === "drop") {
      // NO clear-on-read: this document may be the DOOMED pre-reload one —
      // mogh's sanitize reload was already initiated when the settlement
      // wrote the flag, and this page can mount (RequireAuth bounces a
      // session-less settled document to /login) and consume the flag
      // milliseconds before the unload commits, destroying the signal for
      // the document the user actually lands in. The ok branch still
      // consumes BEFORE navigating, because THAT redirect would otherwise
      // loop; the drop path has no redirect to loop, so TTL expiry is the
      // cleanup. Freshness gates the toast so a stale flag cannot replay it
      // on later /login visits.
      if (!flagFresh(flag)) return;
      if (!dropNoticeShown) {
        dropNoticeShown = true;
        notifications.show({
          title: "Login succeeded but the session could not be stored",
          message: "Please log in again.",
          color: "red",
          // Mantine defaults to autoClose: 4000 — a silent-failure notice that
          // vanishes by ~T+5s would defeat the silent-drop goal (non-silence) and
          // race verify.mjs's fix-dependent probe (~T+8s). Must-act toasts stay.
          autoClose: false,
        });
      }
      return;
    }
    if (!flagFresh(flag)) return; // stale: ignored (TTL expiry); lingers harmlessly until tab close
    // jwt presence, decode-free, via THE parser for the mogh store
    // (redeem-gate.readMoghStore — one parse, shared with the silent-drop
    // check; a second independent parser here is how two copies drift).
    // The exchanged jwt itself is not available post-reload — presence plus
    // the fresh flag is the late-success signal. The fresh "ok" flag is only
    // ever written when the exchange 200'd (cache onSuccess), and a silent
    // drop overwrites it to "drop" in the same dispatch task — so a fresh ok
    // flag certifies the exchanged token itself landed, and the redirect
    // never depended on failure-path store clearing (failure paths are
    // store-neutral). Absent/corrupt store -> false -> no redirect.
    if (hasStoredJwt()) {
      // Consume BEFORE navigating: the next document finds no flag, so the
      // redirect cannot loop — including across StrictMode's double effect.
      consumeRedeemFlag();
      const backto = new URLSearchParams(window.location.search).get("backto");
      window.location.replace(safeLocalPath(backto));
    }
  }, []);

  return (
    <LoginPage
      {...props}
      appName="KOMODO"
      iconLink="/mogh-512x512.png"
      iconLinkAlt="moghtech"
      exampleConfigLink="https://github.com/moghtech/komodo/blob/main/config/core.config.toml"
      onLogin={userInvalidate}
    />
  );
}
