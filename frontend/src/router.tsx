import { Layout } from "@components/layouts";
import { useUser } from "@lib/hooks";
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

export const Router = () => {
  const { data: user, isLoading, error } = useUser();

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
          <Route path="/" element={<Layout />}>
            <Route path="" element={<Home />} />
            <Route path="settings" element={<Settings />} />
            <Route path="tree" element={<Tree />} />
            <Route path="alerts" element={<AlertsPage />} />
            <Route path="updates" element={<UpdatesPage />} />
            <Route path="containers" element={<ContainersPage />} />
            <Route path="resources" element={<AllResources />} />
            <Route path="schedules" element={<SchedulesPage />} />
            <Route path="user-groups/:id" element={<UserGroupPage />} />
            <Route path="users/:id" element={<UserPage />} />
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
