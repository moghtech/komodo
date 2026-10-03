// Adds DELAY_AUTH_MS to /auth/login/* only. Caddy routes that path here.
//
// WHY a real awaited setTimeout: `ClientRequest.setTimeout` (the draft's
// first idea) only arms a socket-inactivity timeout EVENT with no listener —
// it delays nothing. The await BEFORE dispatch is the only working delay.
//
// This proxy is a dumb HTTP pipe on purpose: no upgrades (websockets never
// reach it — Caddy routes only /auth/login/* here), no TLS, no pooling.
import http from "node:http";

const DELAY = parseInt(process.env.DELAY_AUTH_MS ?? "0", 10);

http
  .createServer(async (req, res) => {
    if (req.url?.startsWith("/auth/login/") && DELAY > 0) {
      console.log(`${new Date().toISOString()} delay ${DELAY}ms ${req.method} ${req.url}`);
      await new Promise((r) => setTimeout(r, DELAY)); // the ONLY working delay
      // Client gave up during the await (watchdog fired / Playwright timeout):
      // stop here instead of dispatching to core — a dead-request dispatch
      // still consumes one of core's 5 auth rate-limit attempts and would
      // pollute exactly the exchange-counter metric the harness measures.
      if (res.destroyed) return;
    }
    const upstream = http.request(
      {
        host: "core",
        port: 9120,
        path: req.url,
        method: req.method,
        headers: req.headers,
      },
      (r) => {
        res.writeHead(r.statusCode, r.headers);
        r.pipe(res);
      },
    );
    // Without these, a client abort or core refusal is an unhandled
    // 'error' event → process exit → the knob silently disappears mid-suite.
    req.on("error", () => res.destroy());
    upstream.on("error", () => res.destroy());
    req.pipe(upstream);
  })
  .listen(9999, () => console.log(`delay proxy on :9999 (DELAY_AUTH_MS=${DELAY})`));
