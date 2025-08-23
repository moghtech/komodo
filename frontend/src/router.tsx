import { Layout } from "@components/layouts";
import { LOGIN_TOKENS, useAuth, useUser } from "@lib/hooks";
import UpdatePage from "@pages/update";
import { Loader2 } from "lucide-react";
import { lazy, Suspense } from "react";
import { BrowserRouter, Route, Routes } from "react-router-dom";

// Lazy import pages
const Resources = lazy(() => import("@pages/resources"));
const Resource = lazy(() => import("@pages/resource"));
const Login = lazy(() => import("@pages/login"));
const Tree = lazy(() => import("@pages/home/tree"));
const UpdatesPage = lazy(() => import("@pages/updates"));
const AllResources = lazy(() => import("@pages/home/all_resources"));
const UserDisabled = lazy(() => import("@pages/user_disabled"));
const Home = lazy(() => import("@pages/home"));
const AlertsPage = lazy(() => import("@pages/alerts"));
const UserPage = lazy(() => import("@pages/user"));
const UserGroupPage = lazy(() => import("@pages/user-group"));
const Settings = lazy(() => import("@pages/settings"));
const StackServicePage = lazy(() => import("@pages/stack-service"));
const NetworkPage = lazy(() => import("@pages/server-info/network"));
const ImagePage = lazy(() => import("@pages/server-info/image"));
const VolumePage = lazy(() => import("@pages/server-info/volume"));
const ContainerPage = lazy(() => import("@pages/server-info/container"));
const ContainersPage = lazy(() => import("@pages/containers"));
const SchedulesPage = lazy(() => import("@pages/schedules"));

const sanitize_query = (search: URLSearchParams) => {
  search.delete("token");
  const query = search.toString();
  location.replace(
    `${location.origin}${location.pathname}${query.length ? "?" + query : ""}`
  );
};

let exchange_token_sent = false;

/// returns whether to show login / loading screen depending on state of exchange token loop
const useExchangeToken = () => {
  const search = new URLSearchParams(location.search);
  const exchange_token = search.get("token");
  const { mutate } = useAuth("ExchangeForJwt", {
    onSuccess: ({ user_id, jwt }) => {
      LOGIN_TOKENS.add_and_change(user_id, jwt);
      sanitize_query(search);
    },
  });

  // In this case, failed to get user (jwt unset / invalid)
  // and the exchange token is not in url.
  // Just show the login.
  if (!exchange_token) return false;

  // guard against multiple reqs sent
  // maybe isPending would do this but not sure about with render loop, this for sure will.
  if (!exchange_token_sent) {
    mutate({ token: exchange_token });
    exchange_token_sent = true;
  }

  return true;
};

export const Router = () => {
  const { data: user, isLoading, error } = useUser();

  // Handle exchange token loop to avoid showing login flash
  const exchangeTokenPending = useExchangeToken();
  if (exchangeTokenPending) {
    return (
      <div className="w-screen h-screen flex justify-center items-center">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  if (isLoading && !user) return null;

  if (!user || error) return <Login />;
  if (!user.enabled) return <UserDisabled />;

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
          <Route path="/" element={<Layout />}>
            <Route path="" element={<Home />} />
            <Route path="settings" element={<Settings />} />
            <Route path="tree" element={<Tree />} />
            <Route path="containers" element={<ContainersPage />} />
            <Route path="resources" element={<AllResources />} />
            <Route path="schedules" element={<SchedulesPage />} />
            <Route path="alerts" element={<AlertsPage />} />
            <Route path="user-groups/:id" element={<UserGroupPage />} />
            <Route path="users/:id" element={<UserPage />} />
            <Route path="updates">
              <Route path="" element={<UpdatesPage />} />
              <Route path=":id" element={<UpdatePage />} />
            </Route>
            <Route path=":type">
              <Route path="" element={<Resources />} />
              <Route path=":id" element={<Resource />} />
              <Route
                path=":id/service/:service"
                element={<StackServicePage />}
              />
              <Route
                path=":id/container/:container"
                element={<ContainerPage />}
              />
              <Route path=":id/network/:network" element={<NetworkPage />} />
              <Route path=":id/image/:image" element={<ImagePage />} />
              <Route path=":id/volume/:volume" element={<VolumePage />} />
            </Route>
          </Route>
        </Routes>
      </BrowserRouter>
    </Suspense>
  );

  // return <RouterProvider router={ROUTER} />;
};


