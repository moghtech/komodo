import { useEffect, useMemo, useState } from "react";
import {
  useFilterByUpdateAvailable,
  useRead,
  useResourceParamType,
  useSetTitle,
  useTagsFilter,
  useTemplatesQueryBehavior,
  useUser,
} from "@/lib/hooks";
import { ResourceComponents, UsableResource } from "@/resources";
import { Types } from "komodo_client";
import { Page } from "mogh_ui";
import { Group, Pagination, Stack } from "@mantine/core";
import { TableSkeleton } from "mogh_ui";
import TemplateQuerySelector from "@/components/template-query-selector";
import TagsFilter from "@/components/tags/filter";
import ResourceNotFound from "@/resources/not-found";
import ExportToml from "@/components/export-toml";
import ServerShowStats from "@/resources/server/show-stats";
import { SearchInput } from "mogh_ui";
import { LabelledSwitch } from "mogh_ui";

export default function Resources({ _type }: { _type?: UsableResource }) {
  const isAdmin = useUser().data?.admin ?? false;
  const disableNonAdminCreate =
    useRead("GetCoreInfo", {}).data?.disable_non_admin_create ?? true;

  const __type = useResourceParamType()!;
  const type = _type ? _type : __type;

  const name = type === "ResourceSync" ? "Resource Sync" : type;
  useSetTitle(name + "s");

  const [search, setSearch] = useState("");
  const terms = useMemo(
    () =>
      search
        .toLowerCase()
        .split(" ")
        .map((term) => term.trim())
        .filter((term) => term),
    [search],
  );

  const [filterUpdateAvailable, toggleFilterUpdateAvailable] =
    useFilterByUpdateAvailable();

  const tags = useTagsFilter();
  const [templates] = useTemplatesQueryBehavior();
  const query: Types.ResourceQuery<any> = {
    terms,
    tags,
    templates,
    specific:
      type === "Stack" || type === "Deployment"
        ? { update_available: filterUpdateAvailable }
        : undefined,
  };
  const [page, setPage] = useState(0);
  const _resources = useRead(`List${type}s`, { query, page }).data ?? [];

  // Debounce: prevents flashing when typing / fetching.
  // Must also set immediately upon change of resource type
  const [resources, setResources] = useState(_resources);
  useEffect(() => setResources(_resources), [type]);
  useEffect(() => {
    const handler = setTimeout(() => setResources(_resources), 100);
    return () => {
      clearTimeout(handler);
    };
  }, [_resources]);

  const RC = ResourceComponents[type];

  if (!RC) {
    return <ResourceNotFound type={type} />;
  }

  const targets = resources.map((resource) => ({ type, id: resource.id }));

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
            {(isAdmin || !disableNonAdminCreate) && <RC.New />}
            <RC.BatchExecutions />
            {/* PAGINATION (only shown when needed) */}
            {(resources.length >= 100 || page > 0) && (
              <Pagination.Root
                total={resources.length >= 100 ? page + 2 : page + 1}
                value={page + 1}
                onChange={(page) => setPage(page - 1)}
              >
                <Group gap="0.2rem" justify="center">
                  <Pagination.First />
                  <Pagination.Previous />
                  <Pagination.Items />
                  <Pagination.Next />
                </Group>
              </Pagination.Root>
            )}
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

        {resources ? (
          <RC.Table resources={resources ?? []} />
        ) : (
          <TableSkeleton />
        )}
      </Stack>
    </Page>
  );
}
