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
import { ResourceNotFound } from "@/resources/common";
import Page from "@/ui/page";
import { Box, Group, Switch, TextInput } from "@mantine/core";
import { ICONS } from "@/lib/icons";
import TableSkeleton from "@/ui/table-skeleton";

export default function Resources({ _type }: { _type?: UsableResource }) {
  const is_admin = useUser().data?.admin ?? false;
  const disable_non_admin_create =
    useRead("GetCoreInfo", {}).data?.disable_non_admin_create ?? true;
  const __type = useResourceParamType()!;
  const type = _type ? _type : __type;
  const name = type === "ResourceSync" ? "Resource Sync" : type;
  useSetTitle(name + "s");
  const [search, set] = useState("");
  const [filter_update_available, toggle_filter_update_available] =
    useFilterByUpdateAvailable();
  const query =
    type === "Stack" || type === "Deployment"
      ? {
          query: {
            specific: { update_available: filter_update_available },
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

  const Components = ResourceComponents[type];

  if (!Components) {
    return <ResourceNotFound type={type} />;
  }

  const targets = filtered?.map((resource) => ({ type, id: resource.id }));

  return (
    <Page
      title={`${name}s`}
      icon={Components.Icon}
      description={<Components.Description />}
    >
      <Group justify="space-between">
        <Group>
          {(is_admin || !disable_non_admin_create) && <Components.New />}
          <Components.GroupExecutions />
        </Group>

        <Group>
          {(type === "Stack" || type === "Deployment") && (
            <Switch
              label="Pending Update"
              checked={filter_update_available}
              onChange={toggle_filter_update_available}
              opacity={0.7}
              fz="sm"
            />
          )}
          {/* <TemplateQueryBehaviorSelector />
              <TagsFilter /> */}
          <TextInput
            value={search}
            onChange={(e) => set(e.target.value)}
            placeholder="search..."
            leftSection={<ICONS.Search size="0.8rem" />}
            w={{ sm: 200, lg: 300 }}
          />
        </Group>
      </Group>

      <Box
        p="lg"
        pt="0"
        bd="1px solid var(--mantine-color-accent-border-0)"
        bdrs="md"
        w="100%"
        mah="calc(100vh - 25rem)"
        style={{ overflow: "auto" }}
      >
        {filtered ? (
          <Components.Table resources={filtered ?? []} stickyHeader />
        ) : (
          <TableSkeleton />
        )}
      </Box>
    </Page>
  );
}
