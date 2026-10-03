import { lazy, useEffect } from "react";
import {
  BrowserRouter,
  Navigate,
  Outlet,
  Route,
  Routes,
  useLocation,
} from "react-router-dom";
import { notifications } from "@mantine/notifications";
import { LoadingScreen, useAuthState } from "mogh_ui";
import { useUser } from "@/lib/hooks";
import { useRedeemState } from "@/lib/redeem-gate";
import { MoghAuth } from "komodo_client";
import App from "@/app";

const Login = lazy(() => import("@/pages/login"));
const UserDisabled = lazy(() => import("@/pages/user-disabled"));
const Settings = lazy(() => import("@/pages/settings"));
const Updates = lazy(() => import("@/pages/updates"));
const Update = lazy(() => import("@/pages/update"));
const Alerts = lazy(() => import("@/pages/alerts"));
const Alert = lazy(() => import("@/pages/alert"));
const Dashboard = lazy(() => import("@/pages/dashboard"));
const Resources = lazy(() => import("@/pages/resources"));
const Resource = lazy(() => import("@/pages/resource"));
const Profile = lazy(() => import("@/pages/profile"));
const User = lazy(() => import("@/pages/user"));
const UserGroup = lazy(() => import("@/pages/user-group"));
const Schedules = lazy(() => import("@/pages/schedules"));
const Stats = lazy(() => import("@/pages/stats"));
const Terminals = lazy(() => import("@/pages/terminals"));
const Terminal = lazy(() => import("@/pages/terminal"));
const Containers = lazy(() => import("@/pages/containers"));
const Container = lazy(() => import("@/pages/docker/container"));
const Image = lazy(() => import("@/pages/docker/image"));
const Network = lazy(() => import("@/pages/docker/network"));
const Volume = lazy(() => import("@/pages/docker/volume"));
const StackService = lazy(() => import("@/pages/stack-service"));
const SwarmNode = lazy(() => import("@/pages/swarm/node"));
const SwarmStack = lazy(() => import("@/pages/swarm/stack"));
const SwarmService = lazy(() => import("@/pages/swarm/service"));
const SwarmTask = lazy(() => import("@/pages/swarm/task"));
const SwarmConfig = lazy(() => import("@/pages/swarm/config"));
const SwarmSecret = lazy(() => import("@/pages/swarm/secret"));

// One-shot latch for the settled-failed notification, PER DOCUMENT (module
// scope, not a ref: StrictMode's dev double-mount re-runs the effect with a
// fresh component but the same settlement). Two paths re-enter it:
//  - the dev double-mount itself (mount -> effect -> cleanup -> effect), and
//  - a late onMutate after the watchdog already settled (the pre-arm is
//    idempotent, but the real mutation can still flip pending -> settled-failed
//    a second time) — the URL strip is idempotent, the toast is not.
// A NEW document (real retry) legitimately re-arms: only one notice per
// failure, not one per app lifetime.
let failureNoticeShown = false;

export const Router = () => {
  // mogh_ui's useAuthState fires the redeem mutation synchronously in its body
  // during THIS render — the config onMutate in redeem-gate.ts flips the gate
  // to "pending" before the snapshot read below, so render #1 is already the
  // LoadingScreen. mogh_ui's jwt_redeem_ready URL bit is deliberately unused
  // as the gate: after location.replace is initiated, location.search is
  // stale until the new document commits.
  // redeem-gate.ts owns the why.
  const { passkey_pending, totp } = useAuthState();
  const redeemState = useRedeemState();

  useEffect(() => {
    if (redeemState !== "settled-failed") return;
    // NOTE: failure paths never mutate the token store (the earlier
    // remove_all hygiene was removed entirely: a failed redeem must not
    // wipe a pre-existing valid session). This effect is UI convergence
    // only: URL strip + notification. redeem-gate.ts owns the full rationale.
    const url = new URL(window.location.href);
    for (const p of ["redeem_ready", "totp", "passkey"])
      url.searchParams.delete(p);
    window.history.replaceState(null, "", url.pathname + url.search);
    if (!failureNoticeShown) {
      failureNoticeShown = true;
      notifications.show({
        title: "Login didn't complete",
        // Wording stays true on BOTH end states: with no surviving session
        // this lands on /login; with a valid pre-existing session the app
        // renders right here (failure paths are store-neutral) — never
        // claim a destination.
        message: "The login attempt didn't complete — please try again.",
        color: "red",
      });
    }
  }, [redeemState]);

  if (redeemState === "pending") {
    return <LoadingScreen />;
  }

  if (passkey_pending || totp) {
    return <Login passkeyIsPending={passkey_pending} totpIsPending={totp} />;
  }

  return (
    <BrowserRouter>
      <Routes>
        <Route path="login" element={<Login />} />
        <Route element={<RequireAuth />}>
          <Route path="/" element={<App />}>
            <Route path="" element={<Dashboard />} />
            <Route path="containers" element={<Containers />} />
            <Route path="terminals" element={<Terminals />} />
            <Route path="stats" element={<Stats />} />
            <Route path="schedules" element={<Schedules />} />
            <Route path="profile" element={<Profile />} />
            <Route path="settings" element={<Settings />} />
            <Route path="user-groups/:id" element={<UserGroup />} />
            <Route path="users/:id" element={<User />} />
            <Route path="updates">
              <Route path="" element={<Updates />} />
              <Route path=":id" element={<Update />} />
            </Route>
            <Route path="alerts">
              <Route path="" element={<Alerts />} />
              <Route path=":id" element={<Alert />} />
            </Route>
            <Route path=":type">
              <Route path="" element={<Resources />} />
              <Route path=":id" element={<Resource />} />

              {/* Stack Service */}
              <Route path=":id/service/:service" element={<StackService />} />

              {/* Docker Resource */}
              <Route path=":id/container/:container" element={<Container />} />
              <Route path=":id/network/:network" element={<Network />} />
              <Route path=":id/image/:image" element={<Image />} />
              <Route path=":id/volume/:volume" element={<Volume />} />

              {/* Swarm Resource */}
              <Route path=":id/swarm-node/:node" element={<SwarmNode />} />
              <Route path=":id/swarm-stack/:stack" element={<SwarmStack />} />
              <Route
                path=":id/swarm-service/:service"
                element={<SwarmService />}
              />
              <Route path=":id/swarm-task/:task" element={<SwarmTask />} />
              <Route
                path=":id/swarm-config/:config"
                element={<SwarmConfig />}
              />
              <Route
                path=":id/swarm-secret/:secret"
                element={<SwarmSecret />}
              />

              {/* Terminal Pages */}
              <Route path=":id/terminal/:terminal" element={<Terminal />} />
              <Route
                path=":id/service/:service/terminal/:terminal"
                element={<Terminal />}
              />
              <Route
                path=":id/container/:container/terminal/:terminal"
                element={<Terminal />}
              />
            </Route>
          </Route>
        </Route>
      </Routes>
    </BrowserRouter>
  );
};

const RequireAuth = () => {
  const { data: user, error } = useUser();
  const location = useLocation();

  if (
    (error as { error?: TypeError } | undefined)?.error?.message?.startsWith(
      "NetworkError",
    )
  ) {
    // Will just show the spinner without navigate to login,
    // which won't help because its not a login issue.
    return <LoadingScreen />;
  }

  if (!MoghAuth.LOGIN_TOKENS.jwt() || error) {
    if (location.pathname === "/") {
      return <Navigate to="/login" replace />;
    }
    const backto = encodeURIComponent(location.pathname + location.search);
    return <Navigate to={`/login?backto=${backto}`} replace />;
  }

  if (!user) {
    return <LoadingScreen />;
  }

  if (!user.enabled) {
    return <UserDisabled />;
  }

  return <Outlet />;
};
