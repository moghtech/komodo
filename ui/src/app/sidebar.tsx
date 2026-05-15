import { ICONS } from "@/lib/icons";
import { usableResourcePath } from "@/lib/utils";
import { SIDEBAR_RESOURCES } from "@/resources";
import { Button, Divider, ScrollArea, Stack, Text } from "@mantine/core";
import { ReactNode } from "react";
import { Link, useLocation } from "react-router-dom";

const Sidebar = ({ close }: { close: () => void }) => {
  const location = useLocation().pathname;
  const linkProps = { location, close };
  return (
    <Stack justify="space-between" gap="md" h="96%" m="xl" mt="24" mr="md">
      {/* TOP AREA (scrolling) */}
      <ScrollArea>
        <Stack gap="0.15rem" mr="md">
          <SidebarLink
            label="Dashboard"
            icon={<ICONS.Dashboard size="1rem" />}
            to="/"
            {...linkProps}
          />
          <SidebarLink
            label="Containers"
            icon={<ICONS.Container size="1rem" />}
            to="/containers"
            {...linkProps}
          />
          <SidebarLink
            label="Terminals"
            icon={<ICONS.Terminal size="1rem" />}
            to="/terminals"
            {...linkProps}
          />

          <Divider
            label={
              <Text opacity={0.7} size="sm">
                Resources
              </Text>
            }
            my="0.1rem"
          />

          {SIDEBAR_RESOURCES.map((type) => {
            const Icon = ICONS[type];
            return (
              <SidebarLink
                key={type}
                label={type === "ResourceSync" ? "Syncs" : type + "s"}
                icon={<Icon size="1rem" />}
                to={`/${usableResourcePath(type)}`}
                {...linkProps}
              />
            );
          })}

          <Divider
            label={
              <Text opacity={0.7} size="sm">
                Notifications
              </Text>
            }
            my="0.1rem"
          />

          <SidebarLink
            label="Alerts"
            icon={<ICONS.Alert size="1rem" />}
            to="/alerts"
            {...linkProps}
          />
          <SidebarLink
            label="Updates"
            icon={<ICONS.Update size="1rem" />}
            to="/updates"
            {...linkProps}
          />

          <Divider my="xs" />

          <SidebarLink
            label="Schedules"
            icon={<ICONS.Schedule size="1rem" />}
            to="/schedules"
            {...linkProps}
          />
          <SidebarLink
            label="Settings"
            icon={<ICONS.Settings size="1rem" />}
            to="/settings"
            {...linkProps}
          />
        </Stack>
      </ScrollArea>

      {/* BOTTOM AREA */}
      <Stack gap="lg">
        {/* <Button
          onClick={() => nav("/devices")}
          leftSection={<Server size="1rem" />}
          style={{ justifySelf: "flex-end" }}
          fullWidth
        >
          Devices
        </Button> */}
      </Stack>
    </Stack>
  );
};

const SidebarLink = ({
  label,
  icon,
  to,
  location,
  close,
}: {
  label: string;
  icon: ReactNode;
  to: string;
  location: string;
  close: () => void;
}) => {
  return (
    <Button
      variant={
        (to === "/" ? location === "/" : location.startsWith(to))
          ? "default"
          : "subtle"
      }
      component={Link}
      to={to}
      onClick={close}
      leftSection={icon}
      justify="flex-start"
      fullWidth
    >
      {label}
    </Button>
  );
};

export default Sidebar;
