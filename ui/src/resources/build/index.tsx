import { buildStateIntention, hexColorByIntention } from "@/lib/color";
import { useInvalidate, usePermissions, useRead, useWrite } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import EntityHeader from "@/ui/entity-header";
import BuildTable from "./table";
import NewResource from "@/resources/new";
import BuildConfig from "./config";
import DeleteResource from "../delete";
import {
  ActionIcon,
  Badge,
  Box,
  Group,
  HoverCard,
  Stack,
  Text,
} from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { RunBuild } from "./executions";

export function useBuild(id: string | undefined) {
  return useRead("ListBuilds", {}).data?.find((r) => r.id === id);
}

export function useFullBuild(id: string) {
  return useRead("GetBuild", { build: id }).data;
}

export const BuildComponents: RequiredResourceComponents<
  Types.BuildConfig,
  Types.BuildInfo,
  Types.BuildListItemInfo
> = {
  useList: () => useRead("ListBuilds", {}).data,
  useListItem: useBuild,
  useFull: useFullBuild,

  useResourceLinks: (build) => build?.config?.links,

  useDashboardSummaryData: () => {
    const summary = useRead("GetBuildsSummary", {}).data;
    return [
      { title: "Ok", intention: "Good", value: summary?.ok ?? 0 },
      {
        title: "Building",
        intention: "Warning",
        value: summary?.building ?? 0,
      },
      {
        title: "Failed",
        intention: "Critical",
        value: summary?.failed ?? 0,
      },
      {
        title: "Unknown",
        intention: "Unknown",
        value: summary?.unknown ?? 0,
      },
    ];
  },

  Description: () => <>Build container images.</>,

  New: () => <NewResource type="Build" />,

  GroupExecutions: () => <></>,

  Table: BuildTable,

  Icon: ({ id, size = "1rem" }) => {
    const state = useRead("ListBuilds", {}).data?.find((r) => r.id === id)?.info
      .state;
    const color = state && hexColorByIntention(buildStateIntention(state));
    return <ICONS.Build size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const build = useBuild(id);
    return (
      <EntityHeader
        intent={buildStateIntention(build?.info.state)}
        icon={ICONS.Build}
        name={build?.name}
        state={build?.info.state}
        action={<DeleteResource type="Build" id={id} />}
      />
    );
  },

  State: ({ id }) => {
    let state = useBuild(id)?.info.state;
    return <StatusBadge text={state} intent={buildStateIntention(state)} />;
  },

  Info: {
    Hash: ({ id }) => {
      const info = useFullBuild(id)?.info;
      if (!info?.latest_hash) {
        return null;
      }
      const outOfDate = info.built_hash && info.built_hash !== info.latest_hash;
      return (
        <HoverCard position="bottom-start">
          <HoverCard.Target>
            <Box
              px="sm"
              py="0.2rem"
              bdrs="sm"
              style={{
                borderColor: outOfDate
                  ? "var(--mantine-color-yellow-7)"
                  : "var(--mantine-color-accent-border-1)",
                borderStyle: "solid",
                borderWidth: "1px",
                cursor: "pointer",
              }}
            >
              {info.built_hash ? "built" : "latest"}:{" "}
              {info.built_hash || info.latest_hash}
            </Box>
          </HoverCard.Target>
          <HoverCard.Dropdown>
            <Stack>
              <Stack gap="0.2rem">
                <Badge>message</Badge>
                {info.built_message || info.latest_message}
              </Stack>
              {outOfDate && (
                <Stack gap="0.2rem">
                  <Badge
                    variant="secondary"
                    style={{ borderColor: "var(--mantine-color-yellow-7)" }}
                  >
                    latest
                  </Badge>
                  <Group gap="xs">
                    <Text c="dimmed">{info.latest_hash}:</Text>
                    <Text>{info.latest_message}</Text>
                  </Group>
                </Stack>
              )}
            </Stack>
          </HoverCard.Dropdown>
        </HoverCard>
      );
    },
    Refresh: ({ id }) => {
      const inv = useInvalidate();
      const info = useBuild(id)?.info;
      const { canExecute } = usePermissions({ type: "Build", id });
      const { mutate, isPending } = useWrite("RefreshBuildCache", {
        onSuccess: () => {
          inv(["ListBuilds"], ["GetBuild", { build: id }]);
          notifications.show({ message: "Refreshed source file contents" });
        },
      });

      if (
        !canExecute ||
        // Don't show for UI defined, doesn't do anything
        (!info?.files_on_host && !info?.linked_repo && !info?.repo)
      )
        return null;

      return (
        <ActionIcon onClick={() => mutate({ build: id })} loading={isPending}>
          <ICONS.Refresh size="1rem" />
        </ActionIcon>
      );
    },
  },

  Executions: {
    RunBuild,
  },

  Config: BuildConfig,

  Page: {},
};
