import { useState } from "react";
import { useResourceParamType, useSetTitle } from "@/lib/hooks";
import { ResourceComponents, UsableResource } from "@/resources";
import { Types } from "komodo_client";
import { Page } from "mogh_ui";
import { Group } from "@mantine/core";
import ResourceNotFound from "@/resources/not-found";
import ExportToml from "@/components/export-toml";
import ServerShowStats from "@/resources/server/show-stats";
import ResourceTable from "@/resources/table";

export default function Resources({ _type }: { _type?: UsableResource }) {
  const __type = useResourceParamType()!;
  const type = _type ? _type : __type;

  const name = type === "ResourceSync" ? "Resource Sync" : type;
  useSetTitle(name + "s");

  const [query, setQuery] = useState<Types.ResourceQuery<any>>({});

  const RC = ResourceComponents[type];

  if (!RC) {
    return <ResourceNotFound type={type} />;
  }

  return (
    <Page
      title={`${name}s`}
      icon={RC.Icon}
      description={<RC.Description />}
      oppositeTitle={
        <Group w={{ base: "100%", xs: "fit-content" }}>
          {type === "Server" && <ServerShowStats />}
          <ExportToml listQuery={{ type, query }} />
        </Group>
      }
    >
      <ResourceTable
        type={type}
        onQueryChange={setQuery}
        showTagsQuery
        showTemplateQuery
        showAvailableUpdatesQuery
      />
    </Page>
  );
}
