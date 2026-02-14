import { procedureStateIntention, hexColorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import EntityHeader from "@/ui/entity-header";
import ProcedureTable from "./table";
import NewResource from "@/resources/new";
import ProcedureConfig from "./config";
import DeleteResource from "../delete";
import { RunProcedure } from "./executions";

export function useProcedure(id: string | undefined) {
  return useRead("ListProcedures", {}).data?.find((r) => r.id === id);
}

export function useFullProcedure(id: string) {
  return useRead("GetProcedure", { procedure: id }).data;
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
    const summary = useRead("GetProceduresSummary", {}).data;
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

  GroupExecutions: () => <></>,

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
      <EntityHeader
        intent={procedureStateIntention(procedure?.info.state)}
        icon={ICONS.Procedure}
        name={procedure?.name}
        state={procedure?.info.state}
        status={`${procedure?.info.stages} Stage${procedure?.info.stages === 1 ? "" : "s"}`}
        action={<DeleteResource type="Procedure" id={id} />}
      />
    );
  },

  State: ({ id }) => {
    let state = useProcedure(id)?.info.state;
    return <StatusBadge text={state} intent={procedureStateIntention(state)} />;
  },
  Info: {},

  Executions: {
    RunProcedure,
  },

  Config: ProcedureConfig,

  Page: {},
};
