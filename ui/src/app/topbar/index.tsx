import {
  ActionIcon,
  AppShell,
  Box,
  Burger,
  Button,
  Center,
  Group,
  SimpleGrid,
  Text,
} from "@mantine/core";
import { Link } from "react-router-dom";
import { ThemeToggle } from "mogh_ui";
import UserDropdown from "@/app/topbar/user-dropdown";
import TopbarUpdates from "@/app/topbar/updates";
import OmniSearch from "@/app/topbar/omni-search";
import WebsocketStatus from "@/app/topbar/websocket-status";
import { useRead } from "@/lib/hooks";
import TopbarLink from "./link";
import TopbarAlerts from "./alerts";
import KeyboardShortcuts from "./keyboard-shortcuts";

const Topbar = ({
  opened,
  toggle,
}: {
  opened: boolean;
  toggle: () => void;
}) => {
  const version = useRead("GetVersion", {}, { refetchInterval: 30_000 }).data
    ?.version;
  return (
    <AppShell.Header
      renderRoot={(props) => (
        <SimpleGrid cols={{ base: 2, lg: 3 }} {...props} />
      )}
      style={(theme) => ({
        borderColor: theme.colors["accent-border"][1],
      })}
      bg="accent.1"
      pl="1.3rem"
      pr="2rem"
      py="0rem"
    >
      {/** LEFT AREA */}
      <Group gap="xs" wrap="nowrap" w="fit-content">
        <Burger opened={opened} onClick={toggle} hiddenFrom="sm" size="sm" />

        <ActionIcon
          variant="subtle"
          renderRoot={(props) => <Link to="/" {...props} />}
          size="lg"
          hiddenFrom="md"
        >
          <img src="/mogh-512x512.png" width={32} alt="moghtech" />
        </ActionIcon>

        <Button
          variant="subtle"
          renderRoot={(props) => <Link to="/" {...props} />}
          leftSection={
            <img src="/mogh-512x512.png" width={32} alt="moghtech" />
          }
          size="lg"
          visibleFrom="md"
        >
          <Text fz="h2" fw="450" lts="0.1rem">
            KOMODO
          </Text>
        </Button>
      </Group>

      {/** OMNI SEARCH */}
      <Center visibleFrom="lg">
        <OmniSearch />
      </Center>

      {/** RIGHT AREA */}
      <Group gap="0" style={{ justifySelf: "flex-end" }} wrap="nowrap">
        <Box hiddenFrom="lg">
          <OmniSearch />
        </Box>
        <TopbarLink to="/docs">API</TopbarLink>
        <TopbarLink to="https://komo.do/docs/intro">Docs</TopbarLink>
        {version && (
          <TopbarLink to="https://github.com/moghtech/komodo/releases">
            v{version}
          </TopbarLink>
        )}
        <KeyboardShortcuts />
        <WebsocketStatus />
        <TopbarAlerts />
        <TopbarUpdates />
        <ThemeToggle />
        <UserDropdown />
      </Group>
    </AppShell.Header>
  );
};

export default Topbar;
