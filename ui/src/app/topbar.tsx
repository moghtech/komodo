import {
  AppShell,
  Burger,
  Button,
  Center,
  Flex,
  Group,
  SimpleGrid,
  Text,
} from "@mantine/core";
import { useNavigate } from "react-router-dom";
import { ThemeToggle } from "@/ui/theme-toggle";
import { UserDropdown } from "@/components/user-dropdown";
import TopbarUpdates from "@/components/updates/topbar";
import OmniSearch from "@/components/omni-search";

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
        <SimpleGrid cols={{ base: 2, lg: 3 }} {...props} />
      )}
      style={{
        borderColor: "var(--mantine-color-accent-border-0)",
      }}
      bg="accent.0"
      pl="1.3rem"
      pr="2rem"
      py="0rem"
    >
      {/** LEFT AREA */}
      <Group gap="md">
        <Burger opened={opened} onClick={toggle} hiddenFrom="sm" size="sm" />
        <Button
          variant="subtle"
          c="inherit"
          leftSection={
            <img src="/mogh-512x512.png" width={32} alt="moghtech" />
          }
          onClick={() => nav("/")}
          size="lg"
        >
          <Text fz="h2" fw="450" lts="0.1rem">
            KOMODO
          </Text>
        </Button>
      </Group>

      {/** OMNI SEARCH */}
      <Center>
        <OmniSearch />
      </Center>

      {/** RIGHT AREA */}
      <Group gap="0.3rem" style={{ justifySelf: "flex-end" }}>
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
