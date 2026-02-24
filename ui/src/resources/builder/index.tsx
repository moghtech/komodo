import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import ResourceLink from "@/resources/link";
import NewResource from "@/resources/new";
import BuilderTable from "./table";
import { useServer } from "../server";
import { serverStateIntention } from "@/lib/color";
import ResourceHeader from "../header";
import BuilderConfig from "./config";
import BatchExecutions from "@/components/batch-executions";

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

  BatchExecutions: () => <BatchExecutions type="Builder" executions={[]} />,

  Table: BuilderTable,

  Icon: ({ size = "1rem" }) => {
    return <ICONS.Builder size={size} />;
  },

  ResourcePageHeader: ({ id }) => {
    const builder = useBuilder(id);
    const server = useServer(
      builder?.info.builder_type === "Server"
        ? builder.info.instance_type
        : undefined,
    );
    const intent = server?.info.state
      ? serverStateIntention(server.info.state)
      : "Neutral";
    return (
      <ResourceHeader
        type="Builder"
        id={id}
        resource={builder}
        intent={intent}
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
      />
    );
  },

  State: () => null,
  Info: {},

  Executions: {},

  Config: BuilderConfig,

  Page: {},
};
