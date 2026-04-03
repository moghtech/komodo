import { AppShell, Box } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { Suspense } from "react";
import { Outlet } from "react-router-dom";
import Topbar from "@/app/topbar";
import Sidebar from "@/app/sidebar";
import { LoadingScreen } from "mogh_ui";
import UpdateDetails from "@/components/updates/details";
import AlertDetails from "@/components/alerts/details";

export const TOPBAR_HEIGHT = 62;

const App = () => {
  const [opened, { toggle, close }] = useDisclosure();
  return (
    <AppShell
      padding={{ base: "lg", sm: "xl" }}
      header={{ height: TOPBAR_HEIGHT }}
      navbar={{
        width: 240,
        breakpoint: "sm",
        collapsed: { mobile: !opened },
      }}
    >
      <Topbar opened={opened} toggle={toggle} />

      <AppShell.Navbar
        style={(theme) => {
          return {
            borderColor: theme.colors["accent-border"][1],
          };
        }}
      >
        <Sidebar close={close} />
      </AppShell.Navbar>

      <AppShell.Main>
        <Suspense fallback={<LoadingScreen />}>
          <Box px={{ xl: "xl" }}>
            <Outlet />
          </Box>
          <UpdateDetails />
          <AlertDetails />
        </Suspense>
      </AppShell.Main>
    </AppShell>
  );
};

export default App;
