import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import EntityHeader from "@/ui/entity-header";
import NewResource from "@/resources/new";

export const AlerterComponents: RequiredResourceComponents<
  Types.AlerterConfig,
  undefined,
  Types.AlerterListItemInfo
> = {
  useList: () => useRead("ListAlerters", {}).data,
  useListItem: (id) => AlerterComponents.useList()?.find((r) => r.id === id),

  useFull: (id) => useRead("GetAlerter", { alerter: id }).data,

  useResourceLinks: () => undefined,

  useDashboardSummaryData: () => {
    const summary = useRead("GetAlertersSummary", {}).data;
    return [{ intention: "Good", value: summary?.total ?? 0, title: "Total" }];
  },

  Description: () => <>Route alerts to various endpoints.</>,

  New: () => <NewResource type="Alerter" />,

  GroupExecutions: () => <></>,

  Table: ({ resources }) => (
    // <AlerterTable alerters={resources as Types.AlerterListItem[]} />
    <></>
  ),

  Icon: ({ size = "1rem" }) => {
    return <ICONS.Alerter size={size} />;
  },

  ResourcePageHeader: ({ id }) => {
    const alerter = AlerterComponents.useListItem(id);
    return (
      <EntityHeader
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
  DangerZone: ({ id }) => <></>,

  Page: {},
};
