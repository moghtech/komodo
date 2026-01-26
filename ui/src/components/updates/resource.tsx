import { useRead } from "@/lib/hooks";
import { getUpdateQuery, usableResourcePath } from "@/lib/utils";
import { Types } from "komodo_client";
import Section from "@/ui/section";
import { ICONS } from "@/lib/icons";
import { ActionIcon, Button, ScrollArea, Stack } from "@mantine/core";
import { Link } from "react-router-dom";
import { UsableResource } from "@/resources";
import UpdateCard from "./card";

export default function ResourceUpdates({ type, id }: Types.ResourceTarget) {
  const deployments = useRead("ListDeployments", {}).data;

  const updates = useRead("ListUpdates", {
    query: getUpdateQuery({ type, id }, deployments),
  }).data;

  // const alerts = useRead("ListAlerts", {
  //   query: getUpdateQuery({ type, id }, deployments),
  // }).data;

  // const openAlerts = alerts?.alerts.filter((alert) => !alert.resolved);

  // const showAlerts = type === "Server";

  return (
    <Section
      title="Updates"
      icon={<ICONS.Update size="1rem" />}
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
        {updates?.updates.slice(0, 10).map((update, i) => (
          <UpdateCard key={update.id} update={update} accent={i % 2 === 0} />
        ))}
        <Button
          variant="light"
          c="inherit"
          leftSection={<ICONS.ExternalLink size="1rem" />}
          component={Link}
          to={`/updates?type=${type}&id=${id}`}
          fullWidth
        >
          Show All
        </Button>
      </ScrollArea>
    </Section>
  );
}
