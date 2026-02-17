import { lazy } from "react";
import {
  BrowserRouter,
  Navigate,
  Outlet,
  Route,
  Routes,
  useLocation,
} from "react-router-dom";
import { useAuthState, useUser } from "@/lib/hooks";
import { MoghAuth } from "komodo_client";
import App from "@/app";
import LoadingScreen from "./ui/loading-screen";

const Login = lazy(() => import("@/pages/login"));
const UserDisabled = lazy(() => import("@/pages/user-disabled"));
const Settings = lazy(() => import("@/pages/settings"));
const Updates = lazy(() => import("@/pages/updates"));
const Alerts = lazy(() => import("@/pages/alerts"));
const Dashboard = lazy(() => import("@/pages/dashboard"));
const Resources = lazy(() => import("@/pages/resources"));
const Resource = lazy(() => import("@/pages/resource"));
const Profile = lazy(() => import("@/pages/profile"));
const Schedules = lazy(() => import("@/pages/schedules"));
const Terminals = lazy(() => import("@/pages/terminals"));
const Terminal = lazy(() => import("@/pages/terminal"));
const Containers = lazy(() => import("@/pages/containers"));
const Container = lazy(() => import("@/pages/docker/container"));
const Image = lazy(() => import("@/pages/docker/image"));
const Network = lazy(() => import("@/pages/docker/network"));
const Volume = lazy(() => import("@/pages/docker/volume"));
const StackService = lazy(() => import("@/pages/stack-service"));
const SwarmNode = lazy(() => import("@/pages/swarm/node"));

export const Router = () => {
  // Handle exchange token loop to avoid showing login flash
  const { jwt_redeem_ready, passkey_pending, totp } = useAuthState();

  if (jwt_redeem_ready) {
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
            <Route path="settings" element={<Settings />} />
            <Route path="updates" element={<Updates />} />
            <Route path="alerts" element={<Alerts />} />
            <Route path="containers" element={<Containers />} />
            <Route path="terminals" element={<Terminals />} />
            <Route path="profile" element={<Profile />} />
            <Route path="schedules" element={<Schedules />} />
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
