import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import ResourceHeader from "@/components/resource-header";
import { ResourceLink } from "@/resources/common";
import NewResource from "@/resources/new";

export const BuilderComponents: RequiredResourceComponents<
  Types.BuilderConfig,
  undefined,
  Types.BuilderListItemInfo
> = {
  useListItem: (id) =>
    useRead("ListBuilders", {}).data?.find((r) => r.id === id),

  useFull: (id) => useRead("GetBuilder", { builder: id }).data,

  useResourceLinks: () => undefined,

  useDashboardSummaryData: () => {
    const summary = useRead("GetBuildersSummary", {}).data;
    return [{ intention: "Good", value: summary?.total ?? 0, title: "Total" }];
  },

  Description: () => <>Build on your servers, or single-use AWS instances.</>,

  New: () => <NewResource type="Builder" />,

  GroupExecutions: () => <></>,

  Table: ({ resources }) => (
    // <BuilderTable builders={resources as Types.BuilderListItem[]} />
    <></>
  ),

  Icon: ({ size = "1rem" }) => {
    return <ICONS.Builder size={size} />;
  },

  ResourcePageHeader: ({ id }) => {
    const builder = BuilderComponents.useListItem(id);
    return (
      <ResourceHeader
        intent="None"
        icon={<BuilderComponents.Icon id={id} size="2rem" />}
        name={builder?.name}
        state={builder?.info.builder_type}
        status={
          builder?.info.builder_type === "Aws" ? (
            builder?.info.instance_type
          ) : builder?.info.builder_type === "Server" &&
            builder.info.instance_type ? (
            <ResourceLink type="Server" id={builder.info.instance_type} />
          ) : undefined
        }
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
