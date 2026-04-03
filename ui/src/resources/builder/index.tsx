import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
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
import { useState } from "react";
import { Select } from "@mantine/core";

export function useBuilder(id: string | undefined, useName?: boolean) {
  return useRead("ListBuilders", {}).data?.find((r) =>
    useName ? r.name === id : r.id === id,
  );
}

export function useFullBuilder(id: string) {
  return useRead("GetBuilder", { builder: id }, { refetchInterval: 30_000 })
    .data;
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
    const summary = useRead(
      "GetBuildersSummary",
      {},
      { refetchInterval: 10_000 },
    ).data;
    return [{ intention: "Good", value: summary?.total ?? 0, title: "Total" }];
  },

  Description: () => <>Build on your servers, or single-use AWS instances.</>,

  New: () => {
    const [type, setType] = useState<Types.BuilderConfig["type"]>("Server");
    return (
      <NewResource<{ type: Types.BuilderConfig["type"]; params: {} }>
        type="Builder"
        config={() => ({ type, params: {} })}
        extraInputs={
          <Select
            w="100%"
            label="Builder Type"
            value={type}
            onChange={(type) =>
              type && setType(type as Types.BuilderConfig["type"])
            }
            data={["Server", "Url", "Aws"]}
          />
        }
      />
    );
  },

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
    const coreVersion = useRead("GetVersion", {}).data?.version;
    const intent = server?.info.state
      ? serverStateIntention(
          server.info.state,
          !!coreVersion &&
            !!server.info.version &&
            coreVersion !== server.info.version,
        )
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
