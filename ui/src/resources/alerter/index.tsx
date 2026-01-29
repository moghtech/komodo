import { hexColorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import ResourceHeader from "@/components/resource-header";
import NewResource from "@/resources/new";

export const AlerterComponents: RequiredResourceComponents<
  Types.AlerterConfig,
  undefined,
  Types.AlerterListItemInfo
> = {
  useListItem: (id) =>
    useRead("ListAlerters", {}).data?.find((r) => r.id === id),

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
      <ResourceHeader
        intent="None"
        icon={<AlerterComponents.Icon id={id} size="2rem" />}
        name={alerter?.name}
        state={alerter?.info.enabled ? "Enabled" : "Disabled"}
        status={alerter?.info.endpoint_type}
      />
    );
  },

  State: () => null,
  Status: {},
  Info: {},

  Executions: {},

  Config: () => <>CONFIG</>,
  DangerZone: ({ id }) => <></>,

  Page: {},
};
