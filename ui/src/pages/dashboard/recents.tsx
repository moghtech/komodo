import {
  Flex,
  Group,
  MantineBreakpoint,
  SimpleGrid,
  Stack,
  Text,
} from "@mantine/core";
import { History } from "lucide-react";
import {
  useDashboardPreferences,
  useNoResources,
  useRead,
  useUser,
} from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { usableResourcePath } from "@/lib/utils";
import { ResourceComponents, UsableResource } from "@/resources";
import { ResourceNameSimple } from "@/resources/common";
import { Link, useNavigate } from "react-router-dom";
import DashboardSummary from "@/components/dashboard-summary";
import FancyCard from "@/ui/fancy-card";
import { TemplateMarker } from "@/components/template-marker";
import Tags from "@/components/tags";

const RecentsDashboard = () => {
  const noResources = useNoResources();
  const user = useUser().data!;
  return (
    <Stack gap="xl">
      {noResources && (
        <Group gap="sm" opacity={0.6}>
          <ICONS.Alert size="1rem" />
          <Text fz="lg">
            No resources found.{" "}
            {user.admin
              ? "To get started, create a server."
              : "Contact an admin for access to resources."}
          </Text>
        </Group>
      )}
      <RecentRow type="Swarm" />
      <RecentRow type="Server" />
      <RecentRow type="Stack" />
      <RecentRow type="Deployment" />
      <RecentRow type="Build" />
      <RecentRow type="Repo" />
      <RecentRow type="Procedure" />
      <RecentRow type="Action" />
      <RecentRow type="ResourceSync" />
    </Stack>
  );
};

export default RecentsDashboard;

const RecentRow = ({ type }: { type: UsableResource }) => {
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
      <Stack
        hiddenFrom="md"
        bd="1px solid var(--mantine-color-accent-border-0)"
        bdrs="md"
      >
        {children}
      </Stack>
      <Flex
        visibleFrom="md"
        bd="1px solid var(--mantine-color-accent-border-0)"
        bdrs="md"
      >
        {children}
      </Flex>
    </>
  );
};

const RecentCard = ({
  type,
  id,
  visibleFrom,
}: {
  type: UsableResource;
  id: string;
  visibleFrom?: MantineBreakpoint;
}) => {
  const nav = useNavigate();
  const Components = ResourceComponents[type];
  const resource = Components.useListItem(id);
  const { preferences } = useDashboardPreferences();

  if (!resource) {
    return null;
  }

  const showServerStats = type === "Server" && preferences.showServerStats;

  return (
    <FancyCard
      renderRoot={(props) => (
        <Stack
          {...props}
          justify="space-between"
          p="md"
          bdrs="md"
          renderRoot={(props) => (
            <Link
              to={`${usableResourcePath(type)}/${id}`}
              style={{ color: "inherit", textDecoration: "none" }}
              {...props}
            />
          )}
        />
      )}
      // onClick={() => nav(`${usableResourcePath(type)}/${id}`)}
      visibleFrom={visibleFrom}
    >
      <Group justify="space-between">
        <Group style={{ textWrap: "nowrap" }} gap="sm">
          <Components.Icon id={id} />
          <ResourceNameSimple type={type} id={id} />
          {resource.template && <TemplateMarker type={type} />}
        </Group>
        {/* {type === "Deployment" && <DeploymentUpdateAvailable id={id} small />}
        {type === "Stack" && <StackUpdateAvailable id={id} small />} */}
      </Group>

      <Group gap="xs">
        <Tags tagIds={resource?.tags} py="0.1rem" />
      </Group>
    </FancyCard>
  );
};
