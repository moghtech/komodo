import { ICONS } from "@/lib/icons";
import { usableResourcePath } from "@/lib/utils";
import { SIDEBAR_RESOURCES } from "@/resources";
import { Button, Divider, Flex, ScrollArea, Text } from "@mantine/core";
import { ReactNode } from "react";
import { useLocation, useNavigate } from "react-router-dom";

const Sidebar = ({ close }: { close: () => void }) => {
  const _nav = useNavigate();
  const nav = (to: string) => {
    close();
    _nav(to);
  };
  const location = useLocation().pathname;
  const linkProps = { nav, location };
  return (
    <Flex direction="column" justify="space-between" gap="lg" h="100%" m="xl">
      {/* TOP AREA (scrolling) */}
      <ScrollArea>
        <Flex direction="column" gap="0.5rem">
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
            my="xs"
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
            my="xs"
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
        </Flex>
      </ScrollArea>

      {/* BOTTOM AREA */}
      <Flex direction="column" gap="lg">
        {/* <Button
          onClick={() => nav("/devices")}
          leftSection={<Server size="1rem" />}
          style={{ justifySelf: "flex-end" }}
          fullWidth
        >
          Devices
        </Button> */}
      </Flex>
    </Flex>
  );
};

const SidebarLink = ({
  label,
  icon,
  to,
  nav,
  location,
}: {
  label: string;
  icon: ReactNode;
  to: string;
  nav: (to: string) => void;
  location: string;
}) => {
  return (
    <Button
      variant={
        (to === "/" ? location === "/" : location.startsWith(to))
          ? "default"
          : "subtle"
      }
      c="inherit"
      onClick={() => nav(to)}
      leftSection={icon}
      justify="flex-start"
      fullWidth
    >
      {label}
    </Button>
  );
};

export default Sidebar;
