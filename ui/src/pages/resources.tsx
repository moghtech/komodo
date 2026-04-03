import { useState } from "react";
import {
  useFilterByUpdateAvailable,
  useFilterResources,
  useRead,
  useResourceParamType,
  useSetTitle,
  useTemplatesQueryBehavior,
  useUser,
} from "@/lib/hooks";
import { ResourceComponents, UsableResource } from "@/resources";
import { Types } from "komodo_client";
import { Page } from "mogh_ui";
import { Group, Stack } from "@mantine/core";
import TableSkeleton from "@/ui/table-skeleton";
import TemplateQuerySelector from "@/components/template-query-selector";
import TagsFilter from "@/components/tags/filter";
import ResourceNotFound from "@/resources/not-found";
import ExportToml from "@/components/export-toml";
import ServerShowStats from "@/resources/server/show-stats";
import SearchInput from "@/ui/search-input";
import LabelledSwitch from "@/ui/labelled-switch";

export default function Resources({ _type }: { _type?: UsableResource }) {
  const is_admin = useUser().data?.admin ?? false;
  const disable_non_admin_create =
    useRead("GetCoreInfo", {}).data?.disable_non_admin_create ?? true;
  const __type = useResourceParamType()!;
  const type = _type ? _type : __type;
  const name = type === "ResourceSync" ? "Resource Sync" : type;
  useSetTitle(name + "s");
  const [search, setSearch] = useState("");
  const [filterUpdateAvailable, toggleFilterUpdateAvailable] =
    useFilterByUpdateAvailable();
  const query =
    type === "Stack" || type === "Deployment"
      ? {
          query: {
            specific: { update_available: filterUpdateAvailable },
          },
        }
      : {};
  const [templatesQueryBehavior] = useTemplatesQueryBehavior();
  const resources = useRead(`List${type}s`, query).data;
  const templatesFilterFn =
    templatesQueryBehavior === Types.TemplatesQueryBehavior.Exclude
      ? (resource: Types.ResourceListItem<unknown>) => !resource.template
      : templatesQueryBehavior === Types.TemplatesQueryBehavior.Only
        ? (resource: Types.ResourceListItem<unknown>) => resource.template
        : () => true;
  const filtered = useFilterResources(resources as any, search).filter(
    templatesFilterFn,
  );

  const RC = ResourceComponents[type];

  if (!RC) {
    return <ResourceNotFound type={type} />;
  }

  const targets = filtered?.map((resource) => ({ type, id: resource.id }));

  return (
    <Page
      title={`${name}s`}
      icon={RC.Icon}
      description={<RC.Description />}
      oppositeTitle={
        <Group w={{ base: "100%", xs: "fit-content" }}>
          {type === "Server" && <ServerShowStats />}
          <ExportToml targets={targets} />
        </Group>
      }
    >
      <Stack>
        <Group justify="space-between" w="100%">
          <Group w={{ base: "100%", xs: "fit-content" }}>
            {(is_admin || !disable_non_admin_create) && <RC.New />}
            <RC.BatchExecutions />
          </Group>

          <Group w={{ base: "100%", xs: "fit-content" }}>
            {(type === "Stack" || type === "Deployment") && (
              <LabelledSwitch
                label="Pending Update"
                checked={filterUpdateAvailable}
                onCheckedChange={toggleFilterUpdateAvailable}
                opacity={0.7}
                fz="sm"
              />
            )}
            <TemplateQuerySelector />
            <TagsFilter />
            <SearchInput value={search} onSearch={setSearch} />
          </Group>
        </Group>

        {filtered ? <RC.Table resources={filtered ?? []} /> : <TableSkeleton />}
      </Stack>
    </Page>
  );
}
