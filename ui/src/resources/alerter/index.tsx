import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import NewResource from "@/resources/new";
import AlerterTable from "./table";
import ResourceHeader from "../header";

export function useAlerter(id: string | undefined) {
  return useRead("ListAlerters", {}).data?.find((r) => r.id === id);
}

export function useFullAlerter(id: string) {
  return useRead("GetAlerter", { alerter: id }).data;
}

export const AlerterComponents: RequiredResourceComponents<
  Types.AlerterConfig,
  undefined,
  Types.AlerterListItemInfo
> = {
  useList: () => useRead("ListAlerters", {}).data,
  useListItem: useAlerter,
  useFull: useFullAlerter,

  useResourceLinks: () => undefined,

  useDashboardSummaryData: () => {
    const summary = useRead("GetAlertersSummary", {}).data;
    return [{ intention: "Good", value: summary?.total ?? 0, title: "Total" }];
  },

  Description: () => <>Route alerts to various endpoints.</>,

  New: () => <NewResource type="Alerter" />,

  GroupExecutions: () => <></>,

  Table: AlerterTable,

  Icon: ({ size = "1rem" }) => {
    return <ICONS.Alerter size={size} />;
  },

  ResourcePageHeader: ({ id }) => {
    const alerter = useAlerter(id);
    return (
      <ResourceHeader
        type="Alerter"
        id={id}
        resource={alerter}
        intent="None"
        icon={ICONS.Alerter}
        name={alerter?.name}
        state={alerter?.info.enabled ? "Enabled" : "Disabled"}
        status={alerter?.info.endpoint_type}
      />
    );
  },

  State: () => null,
  Info: {},

  Executions: {},

  Config: () => <>CONFIG</>,

  Page: {},
};
