import { Layout } from "@components/layouts";
import { useUser } from "@lib/hooks";
import { Login } from "@pages/login";
import { Resource } from "@pages/resource";
import { Resources } from "@pages/resources";
import { RouterProvider, createBrowserRouter } from "react-router-dom";
import { Tree } from "@pages/home/tree";
import { UpdatesPage } from "@pages/updates";
import { AllResources } from "@pages/home/all_resources";
import { UserDisabled } from "@pages/user_disabled";
import { Home } from "@pages/home";
import { AlertsPage } from "@pages/alerts";
import { UserPage } from "@pages/user";
import { UserGroupPage } from "@pages/user-group";
import { Settings } from "@pages/settings";
import { StackServicePage } from "@pages/stack-service";
import { NetworkPage } from "@pages/server-info/network";
import { ImagePage } from "@pages/server-info/image";
import { VolumePage } from "@pages/server-info/volume";
import { ContainerPage } from "@pages/server-info/container";
import { ContainersPage } from "@pages/containers";

const ROUTER = createBrowserRouter([
  {
    path: "/",
    element: <Layout />,
    children: [
      { path: "", element: <Home /> },
      { path: "settings", element: <Settings /> },
      { path: "tree", element: <Tree /> },
      { path: "alerts", element: <AlertsPage /> },
      { path: "updates", element: <UpdatesPage /> },
      { path: "updates", element: <UpdatesPage /> },
      { path: "containers", element: <ContainersPage /> },
      { path: "resources", element: <AllResources /> },
      { path: "user-groups/:id", element: <UserGroupPage /> },
      {
        path: "users",
        children: [{ path: ":id", element: <UserPage /> }],
      },
      {
        path: ":type",
        children: [
          { path: "", element: <Resources /> },
          { path: ":id", element: <Resource /> },
          {
            path: ":id/service/:service",
            element: <StackServicePage />,
          },
          {
            path: ":id/container/:container",
            element: <ContainerPage />,
          },
          {
            path: ":id/network/:network",
            element: <NetworkPage />,
          },
          {
            path: ":id/image/:image",
            element: <ImagePage />,
          },
          {
            path: ":id/volume/:volume",
            element: <VolumePage />,
          },
        ],
      },
    ],
  },
]);

export const Router = () => {
  const { data: user, isLoading, error } = useUser();

  if (isLoading && !user) return null;
  if (!user || error) return <Login />;
  if (!user.enabled) return <UserDisabled />;

  return <RouterProvider router={ROUTER} />;
};
