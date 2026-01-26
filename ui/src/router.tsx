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
import { Center, Loader } from "@mantine/core";
import { MoghAuth } from "komodo_client";
import App from "@/app";

const Login = lazy(() => import("@/pages/login"));
const UserDisabled = lazy(() => import("@/pages/user-disabled"));
const Dashboard = lazy(() => import("@/pages/dashboard"));
const Resources = lazy(() => import("@/pages/resources"));
const Resource = lazy(() => import("@/pages/resource"));

export const Router = () => {
  // Handle exchange token loop to avoid showing login flash
  const { jwt_redeem_ready, passkey_pending, totp } = useAuthState();

  if (jwt_redeem_ready) {
    return (
      <Center mt="30vh">
        <Loader size="xl" />
      </Center>
    );
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
            <Route path="containers" element={<>CONTAINERS</>} />
            <Route path="terminals" element={<>TERMINALS</>} />
            <Route path=":type">
              <Route path="" element={<Resources />} />
              <Route path=":id" element={<Resource />} />
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
    return (
      <Center mt="30vh">
        <Loader size="xl" />
      </Center>
    );
  }

  if (!MoghAuth.LOGIN_TOKENS.jwt() || error) {
    if (location.pathname === "/") {
      return <Navigate to="/login" replace />;
    }
    const backto = encodeURIComponent(location.pathname + location.search);
    return <Navigate to={`/login?backto=${backto}`} replace />;
  }

  if (!user) {
    return (
      <Center>
        <Loader size="xl" />
      </Center>
    );
  }

  if (!user.enabled) {
    return <UserDisabled />;
  }

  return <Outlet />;
};
