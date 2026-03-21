import { buildStateIntention, hexColorByIntention } from "@/lib/color";
import { useInvalidate, usePermissions, useRead, useWrite } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import BuildTable from "./table";
import NewResource from "@/resources/new";
import { ActionIcon, Box, Group, Text } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { RunBuild } from "./executions";
import BuildTabs from "./tabs";
import { useBuilder } from "../builder";
import ResourceLink from "../link";
import FileSource from "@/components/file-source";
import HashCompare from "@/components/hash-compare";
import ResourceHeader from "../header";
import BatchExecutions from "@/components/batch-executions";
import { useState } from "react";
import ResourceSelector from "../selector";

export function useBuild(id: string | undefined) {
  return useRead("ListBuilds", {}).data?.find((r) => r.id === id);
}

export function useFullBuild(id: string) {
  return useRead("GetBuild", { build: id }, { refetchInterval: 30_000 }).data;
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

  New: ({ builderId: _builderId }) => {
    const [builderId, setBuilderId] = useState("");
    return (
      <NewResource<Types.BuildConfig>
        type="Build"
        config={() => ({ builder_id: _builderId ?? builderId })}
        extraInputs={
          !_builderId && (
            <ResourceSelector
              type="Builder"
              selected={builderId}
              onSelect={setBuilderId}
              targetProps={{ w: "100%", maw: "100%" }}
              width="target"
              position="bottom"
              clearable
            />
          )
        }
        showTemplateSelector={!!_builderId || !builderId}
      />
    );
  },

  BatchExecutions: () => (
    <BatchExecutions
      type="Build"
      executions={[
        ["RunBuild", ICONS.Build],
        ["CancelBuild", ICONS.Cancel],
      ]}
    />
  ),

  Table: BuildTable,

  Icon: ({ id, size = "1rem", noColor }) => {
    const state = useRead("ListBuilds", {}).data?.find((r) => r.id === id)?.info
      .state;
    const color = noColor
      ? undefined
      : state && hexColorByIntention(buildStateIntention(state));
    return <ICONS.Build size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const build = useBuild(id);
    return (
      <ResourceHeader
        type="Build"
        id={id}
        resource={build}
        intent={buildStateIntention(build?.info.state)}
        icon={ICONS.Build}
        name={build?.name}
        state={build?.info.state}
      />
    );
  },

  State: ({ id }) => {
    let state = useBuild(id)?.info.state;
    return <StatusBadge text={state} intent={buildStateIntention(state)} />;
  },

  Info: {
    Builder: ({ id }) => {
      const info = useBuild(id)?.info;
      const builder = useBuilder(info?.builder_id);
      return builder?.id ? (
        <ResourceLink type="Builder" id={builder?.id} />
      ) : (
        <Group gap="xs">
          <ICONS.Builder size="1rem" />
          <Text>Unknown Builder</Text>
        </Group>
      );
    },
    Source: ({ id }) => {
      const info = useBuild(id)?.info;
      return <FileSource info={info} />;
    },
    Branch: ({ id }) => {
      const info = useBuild(id)?.info;
      if (!info || (!info.repo && !info.linked_repo)) {
        return null;
      }
      return (
        <Group gap="xs">
          <ICONS.Branch size="1rem" />
          {info.branch}
        </Group>
      );
    },
    Hash: ({ id }) => {
      const info = useFullBuild(id)?.info;
      if (!info?.latest_hash) {
        return null;
      }
      return (
        <Box>
          <HashCompare
            lastHash={info?.built_hash}
            lastMessage={info?.built_message}
            lastLabel="built"
            latestHash={info?.latest_hash}
            latestMessage={info?.latest_message}
          />
        </Box>
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

  Config: BuildTabs,

  Page: {},
};
