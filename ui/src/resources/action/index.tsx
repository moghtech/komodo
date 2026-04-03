import { actionStateIntention } from "@/lib/color";
import { useExecute, useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import { StatusBadge } from "mogh_ui";
import { Badge, Group, Popover, Text } from "@mantine/core";
import { Clock } from "lucide-react";
import { useDisclosure } from "@mantine/hooks";
import { updateLogToHtml } from "@/lib/utils";
import ConfirmModalWithDisable from "@/components/confirm-modal-with-disable";
import NewResource from "@/resources/new";
import ActionConfig from "./config";
import ActionTable from "./table";
import ResourceHeader from "../header";
import BatchExecutions from "@/components/batch-executions";

export function useAction(id: string | undefined, useName?: boolean) {
  return useRead("ListActions", {}).data?.find((r) =>
    useName ? r.name === id : r.id === id,
  );
}

export function useFullAction(id: string) {
  return useRead("GetAction", { action: id }, { refetchInterval: 30_000 }).data;
}

export const ActionComponents: RequiredResourceComponents<
  Types.ActionConfig,
  {},
  Types.ActionListItemInfo
> = {
  useList: () => useRead("ListActions", {}).data,
  useListItem: useAction,
  useFull: useFullAction,

  useResourceLinks: () => undefined,

  useDashboardSummaryData: () => {
    const summary = useRead(
      "GetActionsSummary",
      {},
      { refetchInterval: 10_000 },
    ).data;
    return [
      { title: "Ok", intention: "Good", value: summary?.ok ?? 0 },
      {
        title: "Running",
        intention: "Warning",
        value: summary?.running ?? 0,
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

  Description: () => <>Custom scripts using the Komodo client.</>,

  New: () => <NewResource type="Action" />,

  BatchExecutions: () => (
    <BatchExecutions type="Action" executions={[["RunAction", ICONS.Run]]} />
  ),

  Table: ActionTable,

  Icon: ({ id, size = "1rem", noColor }) => {
    const state = useRead("ListActions", {}).data?.find((r) => r.id === id)
      ?.info.state;
    const color = noColor ? undefined : state && actionStateIntention(state);
    return <ICONS.Action size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const action = useAction(id) as
      | Types.ResourceListItem<Types.ActionListItemInfo>
      | undefined;
    return (
      <ResourceHeader
        type="Action"
        id={id}
        resource={action}
        intent={actionStateIntention(action?.info.state)}
        icon={({ size }) => <ActionComponents.Icon id={id} size={size} />}
        name={action?.name}
        state={action?.info.state}
      />
    );
  },

  State: ({ id }) => {
    let state = useAction(id)?.info.state;
    return <StatusBadge text={state} intent={actionStateIntention(state)} />;
  },
  Info: {
    Schedule: ({ id }) => {
      const nextScheduledRun = useAction(id)?.info.next_scheduled_run;
      return (
        <Group gap="xs">
          <Clock size="1rem" />
          Next Run:
          <Text fw="bold">
            {nextScheduledRun
              ? new Date(nextScheduledRun).toLocaleString()
              : "Not Scheduled"}
          </Text>
        </Group>
      );
    },
    ScheduleErrors: ({ id }) => {
      const [opened, { close, open }] = useDisclosure(false);
      const error = useAction(id)?.info.schedule_error;

      if (!error) {
        return null;
      }

      return (
        <Popover position="bottom-start" opened={opened}>
          <Popover.Target>
            <Badge color="red" onMouseEnter={open} onMouseLeave={close}>
              Schedule Error
            </Badge>
          </Popover.Target>

          <Popover.Dropdown style={{ pointerEvents: "none" }}>
            <Text
              component="pre"
              dangerouslySetInnerHTML={{
                __html: updateLogToHtml(error),
              }}
              fz="xs"
            />
          </Popover.Dropdown>
        </Popover>
      );
    },
  },

  Executions: {
    RunAction: ({ id }) => {
      const running =
        (useRead(
          "GetActionActionState",
          { action: id },
          { refetchInterval: 5000 },
        ).data?.running ?? 0) > 0;
      const { mutateAsync, isPending } = useExecute("RunAction");
      const action = useAction(id);

      if (!action) {
        return null;
      }

      return (
        <ConfirmModalWithDisable
          icon={<ICONS.Run size="1rem" />}
          confirmText={action.name}
          onConfirm={async () => {
            await mutateAsync({ action: id, args: {} });
          }}
          loading={running || isPending}
        >
          {running ? "Running" : "Run Action"}
        </ConfirmModalWithDisable>
      );
    },
  },

  Config: ActionConfig,

  Page: {},
};
