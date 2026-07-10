import { useCallback, useEffect, useMemo, useState } from "react";
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
import { Page, useDebounce } from "mogh_ui";
import { Group, Stack } from "@mantine/core";
import ListPagination from "@/components/list-pagination";
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

  const [page, setPage] = useState(0);
  const [search, setSearch] = useState("");
  const debouncedSearch = useDebounce(search, 200);
  const terms = useMemo(() => {
    // Set to page 0 whenever search changes
    setPage(0);
    return debouncedSearch
      .toLowerCase()
      .split(" ")
      .map((term) => term.trim())
      .filter((term) => term);
  }, [debouncedSearch]);

  const [filterUpdateAvailable, toggleFilterUpdateAvailable] =
    useFilterByUpdateAvailable();

  const tags = useTagsFilter();
  const [templates] = useTemplatesQueryBehavior();

  // Server side sort, passed up from the table.
  // The sort keys are resource type specific, so ignore
  // any sort captured on a previously selected type.
  const [_sort, _setSort] = useState<{
    type: UsableResource;
    sort_by?: string;
    sort_desc?: boolean;
  }>({ type });
  const sort = _sort.type === type ? _sort : { type };
  const setSort = useCallback(
    (sort: { sort_by?: string; sort_desc?: boolean }) =>
      _setSort({ type, ...sort }),
    [type],
  );

  // Set to page 0 whenever the resource type, any filter,
  // or the sort changes, otherwise the query can point past
  // the last page and come back empty.
  useEffect(() => {
    setPage(0);
  }, [
    type,
    tags,
    templates,
    filterUpdateAvailable,
    sort.sort_by,
    sort.sort_desc,
  ]);

  const query: Types.ResourceQuery<any> = {
    terms,
    tags,
    templates,
    specific:
      type === "Stack" || type === "Deployment"
        ? { update_available: filterUpdateAvailable }
        : undefined,
  };
  const _resources =
    useRead(`List${type}s`, {
      query,
      page,
      sort_by: sort.sort_by as any,
      sort_desc: sort.sort_desc,
    }).data ?? [];

  // Debounce: prevents flashing when typing / fetching.
  // Must also set immediately upon change of resource type
  const [resources, setResources] = useState(_resources);
  useEffect(() => setResources(_resources), [type]);
  useEffect(() => {
    const handler = setTimeout(() => setResources(_resources), 200);
    return () => {
      clearTimeout(handler);
    };
  }, [_resources]);

  const RC = ResourceComponents[type];

  const Table = useMemo(
    () =>
      resources && RC ? (
        <RC.Table resources={resources} onServerSort={setSort} />
      ) : (
        <TableSkeleton />
      ),
    [resources, setSort],
  );

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
      <Stack>
        <Group justify="space-between" w="100%">
          <Group w={{ base: "100%", xs: "fit-content" }}>
            {(isAdmin || !disableNonAdminCreate) && <RC.New />}
            <RC.BatchExecutions />
            <ListPagination
              page={page}
              setPage={setPage}
              count={resources.length}
            />
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

        {Table}
      </Stack>
    </Page>
  );
}
