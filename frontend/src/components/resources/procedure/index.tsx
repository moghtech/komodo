import {
  ActionWithDialog,
  ResourcePageHeader,
  StatusBadge,
} from "@components/util";
import { useExecute, useRead } from "@lib/hooks";
import { RequiredResourceComponents } from "@types";
import { Clock, Route } from "lucide-react";
import { ProcedureConfig } from "./config";
import { ProcedureTable } from "./table";
import { DeleteResource, NewResource } from "../common";
import {
  procedure_state_intention,
  stroke_color_class_by_intention,
} from "@lib/color";
import { cn, updateLogToHtml } from "@lib/utils";
import { Types } from "komodo_client";
import { DashboardPieChart } from "@pages/home/dashboard";
import { GroupActions } from "@components/group-actions";
import { Tooltip, TooltipContent, TooltipTrigger } from "@ui/tooltip";
import { Card } from "@ui/card";

const useProcedure = (id?: string) =>
  useRead("ListProcedures", {}).data?.find((d) => d.id === id);

const ProcedureIcon = ({ id, size }: { id?: string; size: number }) => {
  const state = useProcedure(id)?.info.state;
  const color = stroke_color_class_by_intention(
    procedure_state_intention(state)
  );
  return <Route className={cn(`w-${size} h-${size}`, state && color)} />;
};

export const ProcedureComponents: RequiredResourceComponents = {
  list_item: (id) => useProcedure(id),
  resource_links: () => undefined,

  Description: () => <>Compose Komodo actions together.</>,

  Dashboard: () => {
    const summary = useRead("GetProceduresSummary", {}).data;
    return (
      <DashboardPieChart
        data={[
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
        ]}
      />
    );
  },

  New: () => <NewResource type="Procedure" />,

  GroupActions: () => (
    <GroupActions type="Procedure" actions={["RunProcedure"]} />
  ),

  Table: ({ resources }) => (
    <ProcedureTable procedures={resources as Types.ProcedureListItem[]} />
  ),

  Icon: ({ id }) => <ProcedureIcon id={id} size={4} />,
  BigIcon: ({ id }) => <ProcedureIcon id={id} size={8} />,

  State: ({ id }) => {
    let state = useProcedure(id)?.info.state;
    return (
      <StatusBadge text={state} intent={procedure_state_intention(state)} />
    );
  },

  Status: {},

  Info: {
    Schedule: ({ id }) => {
      const next_scheduled_run = useProcedure(id)?.info.next_scheduled_run;
      return (
        <div className="flex gap-2 items-center">
          <Clock className="w-4 h-4" />
          Next Run:
          <div className="font-bold">
            {next_scheduled_run
              ? new Date(next_scheduled_run).toLocaleString()
              : "Not Scheduled"}
          </div>
        </div>
      );
    },
    ScheduleErrors: ({ id }) => {
      const error = useProcedure(id)?.info.schedule_error;
      if (!error) {
        return null;
      }
      return (
        <Tooltip>
          <TooltipTrigger asChild>
            <Card className="px-3 py-2 bg-destructive/75 hover:bg-destructive transition-colors cursor-pointer">
              <div className="text-sm text-nowrap overflow-hidden overflow-ellipsis">
                Schedule Error
              </div>
            </Card>
          </TooltipTrigger>
          <TooltipContent className="w-[400px]">
            <pre
              dangerouslySetInnerHTML={{
                __html: updateLogToHtml(error),
              }}
              className="max-h-[500px] overflow-y-auto"
            />
          </TooltipContent>
        </Tooltip>
      );
    },
  },

  Actions: {
    RunProcedure: ({ id }) => {
      const running = useRead(
        "GetProcedureActionState",
        { procedure: id },
        { refetchInterval: 5000 }
      ).data?.running;
      const { mutate, isPending } = useExecute("RunProcedure");
      const procedure = useProcedure(id);
      if (!procedure) return null;
      return (
        <ActionWithDialog
          name={procedure.name}
          title={running ? "Running" : "Run Procedure"}
          icon={<Route className="h-4 w-4" />}
          onClick={() => mutate({ procedure: id })}
          disabled={running || isPending}
          loading={running}
        />
      );
    },
  },

  Page: {},

  Config: ProcedureConfig,

  DangerZone: ({ id }) => <DeleteResource type="Procedure" id={id} />,

  ResourcePageHeader: ({ id }) => {
    const procedure = useProcedure(id);

    return (
      <ResourcePageHeader
        intent={procedure_state_intention(procedure?.info.state)}
        icon={<ProcedureIcon id={id} size={8} />}
        type="Procedure"
        id={id}
        name={procedure?.name}
        state={procedure?.info.state}
        status={`${procedure?.info.stages} Stage${procedure?.info.stages === 1 ? "" : "s"}`}
      />
    );
  },
};
