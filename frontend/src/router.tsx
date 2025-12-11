import { Layout } from "@components/layouts";
import { LOGIN_TOKENS, useAuth, useUser } from "@lib/hooks";
import { preparePasskeyCredential } from "@lib/utils";
import { sanitize_query_inner } from "@main";
import { useToast } from "@ui/use-toast";
import { Types } from "komodo_client";
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

let jwt_redeem_sent = false;
let passkey_sent = false;

/// returns whether to show login / loading screen depending on state of exchange token loop
const useQueryState = () => {
  const { toast } = useToast();
  const onSuccess = ({ user_id, jwt }: Types.JwtResponse) => {
    LOGIN_TOKENS.add_and_change(user_id, jwt);
    sanitize_query_inner(search);
  };
  const { mutate: redeemJwt, isPending: redeem_pending } = useAuth(
    "ExchangeForJwt",
    {
      onSuccess,
    }
  );
  const { mutate: completePasskeyLogin } = useAuth("CompletePasskeyLogin", {
    onSuccess,
  });
  const search = new URLSearchParams(location.search);

  const _passkey = search.get("passkey");
  const passkey = _passkey ? JSON.parse(_passkey) : null;

  // guard against multiple reqs sent
  // maybe isPending would do this but not sure about with render loop, this for sure will.
  if (passkey && !passkey_sent) {
    navigator.credentials
      .get(preparePasskeyCredential(passkey))
      .then((credential) => completePasskeyLogin({ credential }))
      .catch((e) => {
        console.error(e);
        toast({
          title: "Failed to select passkey",
          description: "See console for details",
          variant: "destructive",
        });
      });
    passkey_sent = true;
  }

  const jwt_redeem_ready = search.get("redeem_ready") === "true";

  // guard against multiple reqs sent
  // maybe isPending would do this but not sure about with render loop, this for sure will.
  if (jwt_redeem_ready && !jwt_redeem_sent) {
    redeemJwt({});
    jwt_redeem_sent = true;
  }

  return {
    jwt_redeem_ready,
    redeem_pending,
    passkey_pending: !!passkey,
    totp: search.get("totp") === "true",
  };
};

export const Router = () => {
  // Handle exchange token loop to avoid showing login flash
  const { redeem_pending, passkey_pending, totp } = useQueryState();

  if (totp) {
    return <Login totpIsPending />;
  }

  if (redeem_pending || passkey_pending) {
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
