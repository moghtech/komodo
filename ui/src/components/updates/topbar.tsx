import { useRead, useUser, useUserInvalidate, useWrite } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { ActionIcon, Center, Menu } from "@mantine/core";
import { Circle } from "lucide-react";
import UpdateList from "./list";
import { useDisclosure } from "@mantine/hooks";

export default function TopbarUpdates() {
  const [opened, { open, close }] = useDisclosure();
  const updates = useRead("ListUpdates", {}).data;

  const lastOpened = useUser().data?.last_update_view;
  const unseenUpdate = updates?.updates.some(
    (u) => u.start_ts > (lastOpened ?? Number.MAX_SAFE_INTEGER),
  );

  const userInvalidate = useUserInvalidate();
  const { mutate: setLastSeenUpdate } = useWrite("SetLastSeenUpdate", {
    onSuccess: userInvalidate,
  });

  return (
    <Menu
      position="bottom-end"
      offset={19}
      onOpen={() => {
        open();
        setLastSeenUpdate({});
      }}
      onClose={() => close()}
    >
      <Menu.Target>
        <ActionIcon variant="subtle" c="inherit" size="lg">
          <Center pos="relative">
            <ICONS.Update size="1.3rem" />
            <Circle
              size="0.5rem"
              style={{
                position: "absolute",
                top: "0.5rem",
                right: "0.5rem",
                opacity: unseenUpdate ? 1 : 0,
                transition: "all 150ms ease",
              }}
              color="blue"
            />
          </Center>
        </ActionIcon>
      </Menu.Target>
      <Menu.Dropdown hidden={!opened}>
        <UpdateList
          showAllLink="/updates"
          onUpdateClick={close}
          h={500}
          mah="calc(100vh-90px)"
          w={{ base: "96vw", md: 500, xl3: 600 }}
          large
        />
      </Menu.Dropdown>
    </Menu>
  );
}
