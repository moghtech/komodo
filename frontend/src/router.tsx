import { Layout } from "@components/layouts";
import { LOGIN_TOKENS, useAuth, useUser } from "@lib/hooks";
import { sanitize_query_inner } from "@main";
import { Loader2 } from "lucide-react";
import { lazy, Suspense } from "react";
import {
  BrowserRouter,
  Navigate,
  Outlet,
  Route,
  Routes,
  useLocation,
} from "react-router-dom";

// Lazy import pages
const Dashboard = lazy(() => import("@pages/dashboard"));
const Resources = lazy(() => import("@pages/resources"));
const Resource = lazy(() => import("@pages/resource"));
const Login = lazy(() => import("@pages/login"));
const UpdatesPage = lazy(() => import("@pages/updates"));
const UpdatePage = lazy(() => import("@pages/update"));
const UserDisabled = lazy(() => import("@pages/user_disabled"));
const AlertsPage = lazy(() => import("@pages/alerts"));
const UserPage = lazy(() => import("@pages/user"));
const UserGroupPage = lazy(() => import("@pages/user-group"));
const Settings = lazy(() => import("@pages/settings"));
const StackServicePage = lazy(() => import("@pages/stack-service"));
const NetworkPage = lazy(() => import("@pages/docker/network"));
const ImagePage = lazy(() => import("@pages/docker/image"));
const VolumePage = lazy(() => import("@pages/docker/volume"));
const ContainerPage = lazy(() => import("@pages/docker/container"));
const ContainersPage = lazy(() => import("@pages/containers"));
const TerminalsPage = lazy(() => import("@pages/terminals"));
const TerminalPage = lazy(() => import("@pages/terminal"));
const SchedulesPage = lazy(() => import("@pages/schedules"));
const SwarmNodePage = lazy(() => import("@pages/swarm/node"));
const SwarmServicePage = lazy(() => import("@pages/swarm/service"));
const SwarmTaskPage = lazy(() => import("@pages/swarm/task"));
const SwarmSecretPage = lazy(() => import("@pages/swarm/secret"));
const SwarmConfigPage = lazy(() => import("@pages/swarm/config"));
const SwarmStackPage = lazy(() => import("@pages/swarm/stack"));

let exchange_token_sent = false;

/// returns whether to show login / loading screen depending on state of exchange token loop
const useQueryToken = () => {
  const search = new URLSearchParams(location.search);
  const twoFactorToken = search.get("two_factor");
  const exchangeToken = search.get("token");
  const { mutate } = useAuth("ExchangeForJwt", {
    onSuccess: ({ user_id, jwt }) => {
      LOGIN_TOKENS.add_and_change(user_id, jwt);
      sanitize_query_inner(search);
    },
  });

  if (twoFactorToken) return { twoFactorToken, exchangeTokenPending: false };

  // In this case, failed to get user (jwt unset / invalid)
  // and the exchange token is not in url.
  // Just show the login.
  if (!exchangeToken)
    return { twoFactorToken: "", exchangeTokenPending: false };

  // guard against multiple reqs sent
  // maybe isPending would do this but not sure about with render loop, this for sure will.
  if (!exchange_token_sent) {
    mutate({ token: exchangeToken });
    exchange_token_sent = true;
  }

  return { twoFactorToken: "", exchangeTokenPending: true };
};

export const Router = () => {
  // Handle exchange token loop to avoid showing login flash
  const { twoFactorToken, exchangeTokenPending } = useQueryToken();

  if (twoFactorToken) {
    return <Login twoFactorToken={twoFactorToken} />;
  }

  if (exchangeTokenPending) {
    return (
      <div className="w-screen h-screen flex justify-center items-center">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  return (
    <Suspense
      fallback={
        <div className="w-[100vw] h-[100vh] flex items-center justify-center">
          <Loader2 className="w-16 h-16 animate-spin" />
        </div>
      }
    >
      <BrowserRouter>
        <Routes>
          <Route path="login" element={<Login />} />
          <Route element={<RequireAuth />}>
            <Route path="/" element={<Layout />}>
              <Route path="" element={<Dashboard />} />
              <Route path="settings" element={<Settings />} />
              <Route path="containers" element={<ContainersPage />} />
              <Route path="schedules" element={<SchedulesPage />} />
              <Route path="terminals" element={<TerminalsPage />} />
              <Route path="alerts" element={<AlertsPage />} />
              <Route path="user-groups/:id" element={<UserGroupPage />} />
              <Route path="users/:id" element={<UserPage />} />
              {/* Updates */}
              <Route path="updates">
                <Route path="" element={<UpdatesPage />} />
                <Route path=":id" element={<UpdatePage />} />
              </Route>
              <Route path=":type">
                <Route path="" element={<Resources />} />
                <Route path=":id" element={<Resource />} />
                {/* Stack Service */}
                <Route
                  path=":id/service/:service"
                  element={<StackServicePage />}
                />
                {/* Docker Resource */}
                <Route
                  path=":id/container/:container"
                  element={<ContainerPage />}
                />
                <Route path=":id/network/:network" element={<NetworkPage />} />
                <Route path=":id/image/:image" element={<ImagePage />} />
                <Route path=":id/volume/:volume" element={<VolumePage />} />
                {/* Swarm Resource */}
                <Route
                  path=":id/swarm-node/:node"
                  element={<SwarmNodePage />}
                />
                <Route
                  path=":id/swarm-service/:service"
                  element={<SwarmServicePage />}
                />
                <Route
                  path=":id/swarm-task/:task"
                  element={<SwarmTaskPage />}
                />
                <Route
                  path=":id/swarm-secret/:secret"
                  element={<SwarmSecretPage />}
                />
                <Route
                  path=":id/swarm-config/:config"
                  element={<SwarmConfigPage />}
                />
                <Route
                  path=":id/swarm-stack/:stack"
                  element={<SwarmStackPage />}
                />
                {/* Terminal Page */}
                <Route
                  path=":id/terminal/:terminal"
                  element={<TerminalPage />}
                />
                <Route
                  path=":id/service/:service/terminal/:terminal"
                  element={<TerminalPage />}
                />
                <Route
                  path=":id/container/:container/terminal/:terminal"
                  element={<TerminalPage />}
                />
              </Route>
            </Route>
          </Route>
        </Routes>
      </BrowserRouter>
    </Suspense>
  );

  // return <RouterProvider router={ROUTER} />;
};

const RequireAuth = () => {
  const { data: user, error } = useUser();
  const location = useLocation();

  if (
    (error as { error?: TypeError } | undefined)?.error?.message?.startsWith(
      "NetworkError"
    )
  ) {
    // Will just show the spinner without navigate to login,
    // which won't help because its not a login issue.
    return (
      <div className="w-screen h-screen flex justify-center items-center">
        <Loader2 className="w-16 h-16 animate-spin" />
      </div>
    );
  }

  if (!LOGIN_TOKENS.jwt() || error) {
    if (location.pathname === "/") {
      return <Navigate to="/login" replace />;
    }
    const backto = encodeURIComponent(location.pathname + location.search);
    return <Navigate to={`/login?backto=${backto}`} replace />;
  }

  if (!user) {
    return (
      <div className="w-screen h-screen flex justify-center items-center">
        <Loader2 className="w-16 h-16 animate-spin" />
      </div>
    );
  }

  if (!user.enabled) return <UserDisabled />;

  return <Outlet />;
};
