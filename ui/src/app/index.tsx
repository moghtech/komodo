import { AppShell, Center, Loader } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { Suspense } from "react";
import { Outlet } from "react-router-dom";
import Topbar from "@/app/topbar";
import Sidebar from "@/app/sidebar";

const App = () => {
  const [opened, { toggle, close }] = useDisclosure();
  return (
    <AppShell
      padding="xl"
      header={{ height: 70 }}
      navbar={{
        width: 240,
        breakpoint: "sm",
        collapsed: { mobile: !opened },
      }}
    >
      <Topbar opened={opened} toggle={toggle} />

      <AppShell.Navbar
        style={{
          borderColor: "var(--mantine-color-accent-border-0)",
        }}
      >
        <Sidebar close={close} />
      </AppShell.Navbar>

      <AppShell.Main>
        <Suspense
          fallback={
            <Center h="70vh">
              <Loader size={60} />
            </Center>
          }
        >
          <Outlet />
        </Suspense>
      </AppShell.Main>
    </AppShell>
  );
};

export default App;
