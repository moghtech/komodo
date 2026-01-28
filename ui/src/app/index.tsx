import { AppShell } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { Suspense } from "react";
import { Outlet } from "react-router-dom";
import Topbar from "@/app/topbar";
import Sidebar from "@/app/sidebar";
import LoadingScreen from "@/ui/loading-screen";

export const TOPBAR_HEIGHT = 70;

const App = () => {
  const [opened, { toggle, close }] = useDisclosure();
  return (
    <AppShell
      padding="xl"
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
          <Outlet />
        </Suspense>
      </AppShell.Main>
    </AppShell>
  );
};

export default App;
