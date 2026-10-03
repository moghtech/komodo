// Minimal OIDC provider for the komodo redeem-race harness (moghtech/komodo#1665).
// Issues authorization codes for a static user; auto-approves consent;
// exposes /verify for Caddy forward_auth (session cookie check).
//
// SECURITY BOUNDARY: this mock auto-consents ANY caller (any login/password
// grants a session) — it must never be exposed beyond localhost / the compose
// network (hence the 127.0.0.1 port binding in oidc-dev.compose.yaml).
//
// API notes verified against the pinned oidc-provider@9.12.2 source
// (an earlier draft was written against a version string that does not exist
// on npm — "11.10.1" — so this file follows v9.12.2's actual API):
//   1. `provider.Session.get(ctx)` accepts EITHER a Koa context (`ctx.oidc`
//      set) OR any object with `.req`/`.res` — see lib/models/session.js:51:
//      `provider.createContext(ctx.req, ctx.res).cookies` (createContext is
//      inherited from Koa, and Provider extends Koa). A bare IncomingMessage
//      has neither shape, so we pass `{ req, res }` — the same shape the
//      documented `provider.interactionDetails(req, res)` API uses internally.
//   2. Session has NO `userId()` method. The persisted login field is
//      `accountId` (set via `loginAccount()` during dev-interaction login);
//      truthiness of it is the logged-in test.
//   3. There is no `provider.callback(req, res)`. Provider extends Koa, so
//      `provider.callback()` IS Koa's `app.callback()` — a factory that
//      returns the `(req, res)` handler. Calling it once and reusing the
//      handler avoids rebuilding the compose chain per request.
import Provider from "oidc-provider";
import crypto from "node:crypto";
import http from "node:http";

const issuer = process.env.MOCK_ISSUER ?? "https://portal.oidctest.localhost";
// Browser-facing origin of the komodo RP. The host port is 8443 because the
// harness host cannot publish 443 (rootless podman) — see README "Deviations".
const komodoOrigin =
  process.env.KOMODO_ORIGIN ?? "https://komodo.oidctest.localhost:8443";

// Observed reality: oidc-provider v9 derives every discovered endpoint URL from
// the REQUEST origin (helpers/oidc_context.js urlFor -> `this.ctx.href`), and
// Koa derives scheme/host from the socket unless app.proxy is set. Behind
// Caddy the socket is plain http, so without proxy=true discovery would hand
// the browser `http://portal...` URLs while the issuer (and this harness's
// published port) are https — the authorize redirect would hit a TLS listener
// in plaintext. proxy=true makes Koa honor Caddy's X-Forwarded-Proto/-Host.
const providerConfig = {
  // Session cookie MUST be a parent-domain cookie: host-only for
  // portal.oidctest.localhost would never reach komodo.oidctest.localhost, so
  // Caddy's forward_auth subrequest would 401 EVERY komodo request, including
  // /auth/oidc/callback — no login could ever complete. This mirrors Authelia,
  // which sets its cookie on the shared parent domain.
  // (Deep-merged over defaults by configuration.js, so httpOnly/sameSite=lax
  // from cookies.long defaults survive.)
  cookies: { long: { domain: "oidctest.localhost" } },
  clients: [
    {
      client_id: "komodo-harness",
      // Throwaway harness secret — never use it outside compose/oidc-dev.
      client_secret: "komodo-harness-secret",
      // Second URI: /dev on the portal itself. The /verify gate's unauth
      // redirect (see VERIFY_LOGIN_URL below) sends users through THIS
      // authorize request, so after the portal login they land back on the
      // portal — NOT on komodo's callback. That matters: a gateway-initiated
      // authorize carries a throwaway PKCE challenge, so a code delivered to
      // komodo's callback could never be exchanged by komodo (verifier
      // mismatch). The real login always starts at komodo's /auth/oidc/login.
      redirect_uris: [
        `${komodoOrigin}/auth/oidc/callback`,
        `${new URL("/dev", issuer).href}`,
      ],
      grant_types: ["authorization_code"],
      response_types: ["code"],
      token_endpoint_auth_method: "client_secret_basic",
      scope: "openid email profile",
    },
  ],
  // The keys of `claims` ARE the scopes the AS supports (v9 validates the
  // requested scope against them — requesting `email` with no `claims.email`
  // key fails with "scope must only contain Authorization Server supported
  // scope values"), so declare every scope the komodo client asks for.
  claims: { profile: ["email", "sub"], email: ["email"] },
  features: { devInteractions: { enabled: true } }, // built-in login/consent forms
  findAccount: async (_ctx, id) => ({
    accountId: id,
    claims: async () => ({ sub: id, email: `${id}@oidctest.local` }),
  }),
  pkce: { required: () => true },
  // Auto-approve consent so the Playwright drive is deterministic. loadExistingGrant
  // is called from the loadGrant middleware (actions/authorization/session.js) with
  // the Koa ctx — account/client live under ctx.oidc (NOT ctx.account/ctx.client,
  // which are undefined there and 500 the resume step). Returning a grant that
  // already contains every requested scope is what skips the consent prompt.
  async loadExistingGrant(ctx) {
    const grant = new (provider.Grant)({
      accountId: ctx.oidc.account.accountId,
      clientId: ctx.oidc.client.clientId,
    });
    grant.addOIDCScope("openid email profile");
    await grant.save();
    return grant;
  },
};

const provider = new Provider(issuer, providerConfig);
provider.proxy = true;

// Koa's callback() returns the request handler; build it once.
const providerHandler = provider.callback();

// Unauthenticated-navigation redirect target for /verify (forward_auth gate).
// A fixed throwaway PKCE pair: the gateway-initiated authorize can never
// complete komodo's token exchange (komodo mints its own verifier), so the
// challenge only needs to be a well-formed S256 challenge — it exists to get
// the user onto the dev-interaction login page, where the portal session
// cookie is established.
const THROWAWAY_VERIFIER =
  "harness-throwaway-verifier-0123456789abcdefghijklmnopqrstuv";
const THROWAWAY_CHALLENGE = "-PjgkcO7j7VjW0Eq1W1GXzwSQqdb76oR4YVsn1Iu-QQ";
const VERIFY_LOGIN_URL = (() => {
  const u = new URL("/auth", issuer);
  u.search = new URLSearchParams({
    client_id: "komodo-harness",
    response_type: "code",
    scope: "openid email profile",
    redirect_uri: new URL("/dev", issuer).href,
    code_challenge: THROWAWAY_CHALLENGE,
    code_challenge_method: "S256",
    state: "verify-redirect",
    nonce: "verify-redirect",
  }).toString();
  return u.href;
})();
// Boot-time sanity check: the hardcoded challenge must really be S256(verifier).
if (
  THROWAWAY_CHALLENGE !==
  crypto.createHash("sha256").update(THROWAWAY_VERIFIER).digest().toString("base64url")
) {
  throw new Error("THROWAWAY_CHALLENGE does not match THROWAWAY_VERIFIER (S256)");
}

http
  .createServer(async (req, res) => {
    // Malformed request lines (e.g. `GET //`) make new URL() throw; without
    // this guard the rejection is unhandled and Node kills the process
    // (exit 99) — a published port means any scanner could kill the mock
    // mid-login. 400 and keep serving.
    let url;
    try {
      url = new URL(req.url ?? "/", issuer);
    } catch {
      return res.writeHead(400).end("bad request");
    }
    if (url.pathname === "/dev") {
      // Landing for the /verify gate's unauth redirect (redirect_uri of the
      // throwaway authorize). The code query param is minted but unusable by
      // design — this page just confirms the portal session exists.
      return res
        .writeHead(200, { "content-type": "text/plain" })
        .end(
          "harness: portal session established — return to komodo.oidctest.localhost\n",
        );
    }
    if (url.pathname === "/verify") {
      // Honest failures: lookup ERROR is 503 (provider/API problem), absence
      // of session is 401-or-302. Caddy's access log then discriminates them.
      // `{ req, res }` is the raw-pair shape Session.get supports
      // (see module header, note 1). A missing cookie never throws —
      // Session.get returns an empty Session (accountId undefined) — so the
      // catch below really does mean "lookup error", not "no session".
      let session;
      try {
        session = await provider.Session.get({ req, res });
      } catch (e) {
        console.error("verify: session lookup failed", e);
        return res.writeHead(503).end("lookup error");
      }
      if (session && session.accountId) return res.writeHead(200).end("ok");
      // Harness requirement: an unauthenticated BROWSER navigation must be
      // bounced to the portal (a bare 401 would strand the user and mean the
      // gate is misconfigured). Distinguish navigation from everything else
      // by X-Forwarded-Method, which Caddy's forward_auth subrequest always
      // carries — a direct probe (`curl /verify` with no proxy headers) still
      // gets the plain 401 documented for direct probes. Non-navigations (the ExchangeForJwt
      // POST, API calls) also 401 so XHR sees a clean status instead of
      // following a redirect into the portal's HTML.
      const fwdMethod = req.headers["x-forwarded-method"];
      if (fwdMethod && /^(GET|HEAD)$/i.test(String(fwdMethod))) {
        console.warn("verify: no session for navigation, redirecting to portal");
        return res.writeHead(302, { Location: VERIFY_LOGIN_URL }).end();
      }
      console.warn("verify: no session for request");
      return res.writeHead(401).end("unauthorized");
    }
    providerHandler(req, res);
  })
  .listen(3344, () => console.log("oidc-mock on :3344"));
