import { procedureStateIntention, hexColorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import ResourceHeader from "@/components/resource-header";
import ProcedureTable from "./table";
import NewResource from "@/resources/new";
import ProcedureConfig from "./config";

export const ProcedureComponents: RequiredResourceComponents<
  Types.ProcedureConfig,
  undefined,
  Types.ProcedureListItemInfo
> = {
  useList: () => useRead("ListProcedures", {}).data,
  useListItem: (id) => ProcedureComponents.useList()?.find((r) => r.id === id),

  useFull: (id) => useRead("GetProcedure", { procedure: id }).data,

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

  Icon: ({ id, size = "1rem" }) => {
    const state = useRead("ListProcedures", {}).data?.find((r) => r.id === id)
      ?.info.state;
    const color = state && hexColorByIntention(procedureStateIntention(state));
    return <ICONS.Procedure size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const procedure = ProcedureComponents.useListItem(id);
    return (
      <ResourceHeader
        intent={procedureStateIntention(procedure?.info.state)}
        icon={<ProcedureComponents.Icon id={id} size="2rem" />}
        name={procedure?.name}
        state={procedure?.info.state}
        status={`${procedure?.info.stages} Stage${procedure?.info.stages === 1 ? "" : "s"}`}
      />
    );
  },

  State: ({ id }) => {
    let state = ProcedureComponents.useListItem(id)?.info.state;
    return <StatusBadge text={state} intent={procedureStateIntention(state)} />;
  },
  Info: {},

  Executions: {},

  Config: ProcedureConfig,
  DangerZone: ({ id }) => <></>,

  Page: {},
};
