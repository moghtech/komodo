import { procedureStateIntention, hexColorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import ProcedureTable from "./table";
import NewResource from "@/resources/new";
import ProcedureConfig from "./config";
import { RunProcedure } from "./executions";
import ResourceHeader from "../header";
import BatchExecutions from "@/components/batch-executions";
import { Badge, Group, Popover, Text } from "@mantine/core";
import { Clock } from "lucide-react";
import { useDisclosure } from "@mantine/hooks";
import { updateLogToHtml } from "@/lib/utils";

export function useProcedure(id: string | undefined, useName?: boolean) {
  return useRead("ListProcedures", {}).data?.find((r) =>
    useName ? r.name === id : r.id === id,
  );
}

export function useFullProcedure(id: string) {
  return useRead("GetProcedure", { procedure: id }, { refetchInterval: 30_000 })
    .data;
}

export const ProcedureComponents: RequiredResourceComponents<
  Types.ProcedureConfig,
  undefined,
  Types.ProcedureListItemInfo
> = {
  useList: () => useRead("ListProcedures", {}).data,
  useListItem: useProcedure,
  useFull: useFullProcedure,

  useResourceLinks: () => undefined,

  useDashboardSummaryData: () => {
    const summary = useRead(
      "GetProceduresSummary",
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

  Description: () => <>Orchestrate multiple Komodo executions.</>,

  New: () => <NewResource type="Procedure" />,

  BatchExecutions: () => (
    <BatchExecutions
      type="Procedure"
      executions={[["RunProcedure", ICONS.Run]]}
    />
  ),

  Table: ProcedureTable,

  Icon: ({ id, size = "1rem", noColor }) => {
    const state = useRead("ListProcedures", {}).data?.find((r) => r.id === id)
      ?.info.state;
    const color = noColor
      ? undefined
      : state && hexColorByIntention(procedureStateIntention(state));
    return <ICONS.Procedure size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const procedure = useProcedure(id);
    return (
      <ResourceHeader
        type="Procedure"
        id={id}
        resource={procedure}
        intent={procedureStateIntention(procedure?.info.state)}
        icon={ICONS.Procedure}
        name={procedure?.name}
        state={procedure?.info.state}
        status={`${procedure?.info.stages} Stage${procedure?.info.stages === 1 ? "" : "s"}`}
      />
    );
  },

  State: ({ id }) => {
    let state = useProcedure(id)?.info.state;
    return <StatusBadge text={state} intent={procedureStateIntention(state)} />;
  },
  Info: {
    Schedule: ({ id }) => {
      const nextScheduledRun = useProcedure(id)?.info.next_scheduled_run;
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
      const error = useProcedure(id)?.info.schedule_error;

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
    RunProcedure,
  },

  Config: ProcedureConfig,

  Page: {},
};
