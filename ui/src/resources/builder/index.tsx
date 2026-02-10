import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import EntityHeader from "@/ui/entity-header";
import ResourceLink from "@/resources/link";
import NewResource from "@/resources/new";
import DeleteResource from "../delete";

export function useBuilder(id: string | undefined) {
  return useRead("ListBuilders", {}).data?.find((r) => r.id === id);
}

export function useFullBuilder(id: string) {
  return useRead("GetBuilder", { builder: id }).data;
}

export const BuilderComponents: RequiredResourceComponents<
  Types.BuilderConfig,
  undefined,
  Types.BuilderListItemInfo
> = {
  useList: () => useRead("ListBuilders", {}).data,
  useListItem: useBuilder,
  useFull: useFullBuilder,

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
    const builder = useBuilder(id);
    return (
      <EntityHeader
        intent="None"
        icon={ICONS.Builder}
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
        action={<DeleteResource type="Builder" id={id} />}
      />
    );
  },

  State: () => null,
  Info: {},

  Executions: {},

  Config: () => <>CONFIG</>,

  Page: {},
};
