import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { ActionIcon, Box, Center, Menu } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import AlertList from "@/components/alerts/list";
import { Types } from "komodo_client";

const QUERY: Types.ListAlerts["query"] = { resolved: false };

export default function TopbarAlerts() {
  const [opened, { open, close }] = useDisclosure();
  const alerts = useRead(
    "ListAlerts",
    { query: QUERY },
    { refetchInterval: 3_000 },
  ).data;

  if (!alerts?.alerts.length) {
    return null;
  }

  return (
    <Menu
      opened={opened}
      position="bottom-end"
      offset={20}
      onOpen={open}
      onClose={close}
    >
      <Menu.Target>
        <ActionIcon size="xl" variant="subtle">
          <Center pos="relative">
            <ICONS.Alert size="1.3rem" />
            <Box
              bg="red"
              px="0.3rem"
              py="0.1rem"
              fz="xs"
              pos="absolute"
              top="-8px"
              right="-8px"
              bdrs="md"
            >
              {alerts.alerts.length}
            </Box>
          </Center>
        </ActionIcon>
      </Menu.Target>
      <Menu.Dropdown>
        <AlertList
          query={QUERY}
          showAllLink="/alerts"
          onAlertClick={close}
          mah="min(500px, calc(100vh - 90px))"
          w={{ base: "92vw", md: 500, xl3: 600 }}
          large
        />
      </Menu.Dropdown>
    </Menu>
  );
}
