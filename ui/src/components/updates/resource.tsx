import { useRead } from "@/lib/hooks";
import { getUpdateQuery, usableResourcePath } from "@/lib/utils";
import { Types } from "komodo_client";
import Section from "@/ui/section";
import { ICONS } from "@/theme/icons";
import { ActionIcon, ScrollArea, Stack } from "@mantine/core";
import { Link } from "react-router-dom";
import { UsableResource } from "@/resources";
import UpdateList from "./list";
import { useMemo } from "react";

export default function ResourceUpdates({ type, id }: Types.ResourceTarget) {
  const deployments = useRead("ListDeployments", {}).data;

  const query = useMemo(
    () => getUpdateQuery({ type, id }, deployments),
    [type, id, deployments],
  );

  // const alerts = useRead("ListAlerts", {
  //   query: getUpdateQuery({ type, id }, deployments),
  // }).data;

  // const openAlerts = alerts?.alerts.filter((alert) => !alert.resolved);

  // const showAlerts = type === "Server";

  return (
    <Section
      title="Updates"
      icon={<ICONS.Update size="1.3rem" />}
      actions={
        <ActionIcon
          component={Link}
          to={`/${usableResourcePath(type as UsableResource)}/${id}/updates`}
        >
          <ICONS.ExternalLink size="1rem" />
        </ActionIcon>
      }
      maw={{ xl: 500, xl3: 600 }}
      withBorder
    >
      <ScrollArea component={Stack} mah={180} pr="sm">
        <UpdateList
          query={query}
          max={10}
          showAllLink={`/updates?type=${type}&id=${id}`}
        />
      </ScrollArea>
    </Section>
  );
}
