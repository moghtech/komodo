import {
  Box,
  Flex,
  Group,
  MantineBreakpoint,
  SimpleGrid,
  Stack,
} from "@mantine/core";
import { History } from "lucide-react";
import { useDashboardPreferences, useRead, useUser } from "@/lib/hooks";
import { usableResourcePath } from "@/lib/utils";
import {
  ResourceComponents,
  SIDEBAR_RESOURCES,
  UsableResource,
} from "@/resources";
import { Link, useNavigate } from "react-router-dom";
import DashboardSummary from "@/components/dashboard-summary";
import FancyCard from "@/ui/fancy-card";
import { TemplateMarker } from "@/components/template-marker";
import Tags from "@/components/tags";
import ResourceName from "@/resources/name";
import DeploymentUpdateAvailable from "@/resources/deployment/update-available";
import StackUpdateAvailable from "@/resources/stack/update-available";
import DashboardNoResources from "./no-resources";
import ServerStatsCard from "@/resources/server/stats-card";

export default function DashboardRecents() {
  return (
    <Stack gap="xl">
      <DashboardNoResources />
      {SIDEBAR_RESOURCES.map((type) => (
        <RecentRow key={type} type={type} />
      ))}
    </Stack>
  );
}

function RecentRow({ type }: { type: UsableResource }) {
  const nav = useNavigate();
  const _recents = useUser().data?.recents?.[type]?.slice(0, 6);
  const _resources = useRead(`List${type}s`, {}).data;
  const recents = _recents?.filter(
    (recent) => !_resources?.every((resource) => resource.id !== recent),
  );

  const resources = _resources
    ?.filter((r) => !recents?.includes(r.id))
    .map((r) => r.id);

  const ids = [
    ...(recents ?? []),
    ...(resources?.slice(0, 8 - (recents?.length || 0)) ?? []),
  ];

  const Components = ResourceComponents[type];

  const data = Components.useDashboardSummaryData?.();

  if (ids.length === 0) {
    return;
  }

  const name = type === "ResourceSync" ? "Resource Sync" : type;

  const children = (
    <>
      <DashboardSummary
        name={name}
        icon={<Components.Icon />}
        data={data}
        onClick={() => nav(`/${usableResourcePath(type)}`)}
      >
        {Components.DashboardSummary && <Components.DashboardSummary />}
      </DashboardSummary>
      <Stack w="100%" px="lg" py="md">
        <Flex align="center" gap="xs" opacity={0.6}>
          <History size="1.2rem" />
          Recently Viewed
        </Flex>
        <SimpleGrid cols={{ base: 1, lg: 2, xl2: 3, xl4: 4 }}>
          {ids.map((id, i) => (
            <RecentCard
              key={type + id}
              type={type}
              id={id}
              visibleFrom={
                i > 5 ? "xl4" : i > 3 ? "xl2" : i > 1 ? "lg" : undefined
              }
            />
          ))}
        </SimpleGrid>
      </Stack>
    </>
  );

  return (
    <>
      <Stack hiddenFrom="md" className="bordered-light" bdrs="md">
        {children}
      </Stack>
      <Flex visibleFrom="md" className="bordered-light" bdrs="md">
        {children}
      </Flex>
    </>
  );
}

function RecentCard({
  type,
  id,
  visibleFrom,
}: {
  type: UsableResource;
  id: string;
  visibleFrom?: MantineBreakpoint;
}) {
  const Components = ResourceComponents[type];
  const resource = Components.useListItem(id);
  const { preferences } = useDashboardPreferences();

  if (!resource) {
    return null;
  }

  return (
    <FancyCard
      renderRoot={(props) => (
        <Stack
          {...props}
          justify="space-between"
          p="md"
          bdrs="md"
          gap="0"
          renderRoot={(props) => (
            <Link
              to={`${usableResourcePath(type)}/${id}`}
              style={{ color: "inherit", textDecoration: "none" }}
              {...props}
            />
          )}
        />
      )}
      visibleFrom={visibleFrom}
    >
      <Group justify="space-between">
        <Group style={{ textWrap: "nowrap" }} gap="sm">
          <Components.Icon id={id} />
          <ResourceName type={type} id={id} />
          {resource.template && <TemplateMarker type={type} />}
        </Group>
        {type === "Deployment" && <DeploymentUpdateAvailable id={id} small />}
        {type === "Stack" && <StackUpdateAvailable id={id} small />}
      </Group>

      {type === "Server" && (
        <Box
          mt={preferences.showServerStats ? "md" : "0"}
          mah={preferences.showServerStats ? "10rem" : "0rem"}
          opacity={preferences.showServerStats ? 1 : 0}
          style={{
            overflow: "hidden",
            transition: "all 500ms ease-in-out",
          }}
        >
          <ServerStatsCard id={id} />
        </Box>
      )}

      {resource.tags.length ? (
        <Group gap="xs" mt="md">
          <Tags tagIds={resource?.tags} py="0.1rem" />
        </Group>
      ) : undefined}
    </FancyCard>
  );
}
