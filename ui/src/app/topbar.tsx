import { AppShell, Burger, Button, Flex, Group, Text } from "@mantine/core";
import { useNavigate } from "react-router-dom";
import { ThemeToggle } from "@/ui/theme-toggle";
import { UserDropdown } from "@/components/user-dropdown";
import TopbarUpdates from "@/components/updates/topbar";

const Topbar = ({
  opened,
  toggle,
}: {
  opened: boolean;
  toggle: () => void;
}) => {
  const nav = useNavigate();
  return (
    <AppShell.Header
      renderRoot={(props) => (
        <Flex align="center" justify="space-between" {...props} />
      )}
      style={{
        borderColor: "var(--mantine-color-accent-border-0)",
      }}
      bg="accent.0"
      pl="1.3rem"
      pr="2rem"
      py="0rem"
    >
      <Group gap="md">
        <Burger opened={opened} onClick={toggle} hiddenFrom="sm" size="sm" />
        <Button
          variant="subtle"
          c="inherit"
          leftSection={
            <img src="/mogh-512x512.png" width={32} alt="moghtech" />
          }
          onClick={() => nav("/")}
        >
          <Text fz="h2" fw="450" lts="0.1rem">
            KOMODO
          </Text>
        </Button>
      </Group>
      <Group gap="0.3rem">
        <Group gap="0.5rem">
          <TopbarUpdates />
          <ThemeToggle />
        </Group>
        <UserDropdown />
      </Group>
    </AppShell.Header>
  );
};

export default Topbar;
