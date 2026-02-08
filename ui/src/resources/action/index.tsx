import { actionStateIntention, hexColorByIntention } from "@/lib/color";
import { useExecute, useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import EntityHeader from "@/ui/entity-header";
import { Badge, Group, Popover, Text } from "@mantine/core";
import { Clock } from "lucide-react";
import { useDisclosure } from "@mantine/hooks";
import { updateLogToHtml } from "@/lib/utils";
import ConfirmModal from "@/ui/confirm-modal";
import NewResource from "@/resources/new";
import ActionConfig from "./config";
import ActionTable from "./table";
import DeleteResource from "../delete";

export const ActionComponents: RequiredResourceComponents<
  Types.ActionConfig,
  {},
  Types.ActionListItemInfo
> = {
  useList: () => useRead("ListActions", {}).data,
  useListItem: (id) => ActionComponents.useList()?.find((r) => r.id === id),

  useFull: (id) => useRead("GetAction", { action: id }).data,

  useResourceLinks: () => undefined,

  useDashboardSummaryData: () => {
    const summary = useRead("GetActionsSummary", {}).data;
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

  GroupExecutions: () => <></>,

  Table: ActionTable,

  Icon: ({ id, size = "1rem" }) => {
    const state = useRead("ListActions", {}).data?.find((r) => r.id === id)
      ?.info.state;
    const color = state && hexColorByIntention(actionStateIntention(state));
    return <ICONS.Action size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const action = ActionComponents.useListItem(id) as
      | Types.ResourceListItem<Types.ActionListItemInfo>
      | undefined;
    return (
      <EntityHeader
        intent={actionStateIntention(action?.info.state)}
        icon={({ size }) => <ActionComponents.Icon id={id} size={size} />}
        name={action?.name}
        state={action?.info.state}
        action={<DeleteResource type="Action" id={id} />}
      />
    );
  },

  State: ({ id }) => {
    let state = ActionComponents.useListItem(id)?.info.state;
    return <StatusBadge text={state} intent={actionStateIntention(state)} />;
  },
  Info: {
    Schedule: ({ id }) => {
      const next_scheduled_run =
        ActionComponents.useListItem(id)?.info.next_scheduled_run;
      return (
        <Group>
          <Clock size="1rem" />
          Next Run:
          <Text fw="bold">
            {next_scheduled_run
              ? new Date(next_scheduled_run).toLocaleString()
              : "Not Scheduled"}
          </Text>
        </Group>
      );
    },
    ScheduleErrors: ({ id }) => {
      const [opened, { close, open }] = useDisclosure(false);
      const error = ActionComponents.useListItem(id)?.info.schedule_error;

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
      const action = ActionComponents.useListItem(id);

      if (!action) {
        return null;
      }

      return (
        <ConfirmModal
          icon={<ICONS.Action size="1rem" />}
          confirmText={action.name}
          onConfirm={async () => {
            await mutateAsync({ action: id, args: {} });
          }}
          loading={running || isPending}
        >
          {running ? "Running" : "Run Action"}
        </ConfirmModal>
      );
    },
  },

  Config: ActionConfig,

  Page: {},
};
