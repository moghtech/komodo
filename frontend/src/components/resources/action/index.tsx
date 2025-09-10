import { ActionWithDialog, StatusBadge } from "@components/util";
import { useExecute, useRead } from "@lib/hooks";
import { RequiredResourceComponents } from "@types";
import { Clapperboard, Clock } from "lucide-react";
import { ActionConfig } from "./config";
import { ActionTable } from "./table";
import { DeleteResource, NewResource, ResourcePageHeader } from "../common";
import {
  action_state_intention,
  stroke_color_class_by_intention,
} from "@lib/color";
import { cn, updateLogToHtml } from "@lib/utils";
import { Types } from "komodo_client";
import { DashboardPieChart } from "@pages/home/dashboard";
import { GroupActions } from "@components/group-actions";
import { Tooltip, TooltipContent, TooltipTrigger } from "@ui/tooltip";
import { Card } from "@ui/card";

const useAction = (id?: string) =>
  useRead("ListActions", {}).data?.find((d) => d.id === id);

const ActionIcon = ({ id, size }: { id?: string; size: number }) => {
  const state = useAction(id)?.info.state;
  const color = stroke_color_class_by_intention(action_state_intention(state));
  return <Clapperboard className={cn(`w-${size} h-${size}`, state && color)} />;
};

export const ActionComponents: RequiredResourceComponents = {
  list_item: (id) => useAction(id),
  resource_links: () => undefined,

  Description: () => <>Custom scripts using the Komodo client.</>,

  Dashboard: () => {
    const summary = useRead("GetActionsSummary", {}).data;
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

  New: () => <NewResource type="Action" />,

  GroupActions: () => <GroupActions type="Action" actions={["RunAction"]} />,

  Table: ({ resources }) => (
    <ActionTable actions={resources as Types.ActionListItem[]} />
  ),

  Icon: ({ id }) => <ActionIcon id={id} size={4} />,
  BigIcon: ({ id }) => <ActionIcon id={id} size={8} />,

  State: ({ id }) => {
    let state = useAction(id)?.info.state;
    return <StatusBadge text={state} intent={action_state_intention(state)} />;
  },

  Status: {},

  Info: {
    Schedule: ({ id }) => {
      const next_scheduled_run = useAction(id)?.info.next_scheduled_run;
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
      const error = useAction(id)?.info.schedule_error;
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
    RunAction: ({ id }) => {
      const running =
        (useRead(
          "GetActionActionState",
          { action: id },
          { refetchInterval: 5000 }
        ).data?.running ?? 0) > 0;
      const { mutate, isPending } = useExecute("RunAction");
      const action = useAction(id);
      if (!action) return null;
      return (
        <ActionWithDialog
          name={action.name}
          title={running ? "Running" : "Run Action"}
          icon={<Clapperboard className="h-4 w-4" />}
          onClick={() => mutate({ action: id, args: {} })}
          disabled={running || isPending}
          loading={running}
        />
      );
    },
  },

  Page: {},

  Config: ActionConfig,

  DangerZone: ({ id }) => <DeleteResource type="Action" id={id} />,

  ResourcePageHeader: ({ id }) => {
    const action = useAction(id);
    return (
      <ResourcePageHeader
        intent={action_state_intention(action?.info.state)}
        icon={<ActionIcon id={id} size={8} />}
        type="Action"
        id={id}
        resource={action}
        state={action?.info.state}
        status={undefined}
      />
    );
  },
};
