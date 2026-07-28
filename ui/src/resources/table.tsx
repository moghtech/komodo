import { Group, Stack, StackProps } from "@mantine/core";
import {
  RequiredResourceComponents,
  ResourceComponents,
  UsableResource,
} from ".";
import ResourceNotFound from "./not-found";
import ListPagination from "@/components/list-pagination";
import { LabelledSwitch, SearchInput, TableSkeleton } from "mogh_ui";
import TemplateQuerySelector from "@/components/template-query-selector";
import TagsFilter from "@/components/tags/filter";
import { useCallback, useEffect, useMemo, useState } from "react";
import {
  useDebouncedTermSearch,
  useFilterByUpdateAvailable,
  useRead,
  useTagsFilter,
  useTemplatesQueryBehavior,
  useUser,
} from "@/lib/hooks";
import { Types } from "komodo_client";

export interface ResourceTableProps extends StackProps {
  type: UsableResource;
  showTagsQuery?: boolean;
  showTemplateQuery?: boolean;
  showAvailableUpdatesQuery?: boolean;
  onQueryChange?: (query: Types.ResourceQuery<any>) => void;
  newProps?: Parameters<RequiredResourceComponents["New"]>[0];
  specific?: any;
}

export default function ResourceTable({
  type,
  showTagsQuery,
  showTemplateQuery,
  showAvailableUpdatesQuery,
  onQueryChange,
  newProps,
  specific,
  ...stackProps
}: ResourceTableProps) {
  const isAdmin = useUser().data?.admin ?? false;
  const disableNonAdminCreate =
    useRead("GetCoreInfo", {}).data?.disable_non_admin_create ?? true;

  const [page, setPage] = useState(0);

  const { search, setSearch, terms } = useDebouncedTermSearch({
    onUpdate: () => setPage(0),
  });

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

  const query: Types.ResourceQuery<any> = useMemo(() => {
    const query = {
      terms,
      tags: showTagsQuery ? tags : undefined,
      templates: showTemplateQuery ? templates : undefined,
      specific:
        showAvailableUpdatesQuery && (type === "Stack" || type === "Deployment")
          ? { update_available: filterUpdateAvailable, ...specific }
          : specific,
    };
    onQueryChange?.(query);
    return query;
  }, [
    type,
    terms,
    showTagsQuery,
    tags,
    showTemplateQuery,
    templates,
    showAvailableUpdatesQuery,
    filterUpdateAvailable,
    specific,
  ]);

  const resources =
    useRead(
      `List${type}s`,
      {
        query,
        page,
        sort_by: sort.sort_by as any,
        sort_desc: sort.sort_desc,
      },
      {
        refetchInterval: 15_000,
        // Keep the previous rows visible while fetching after a query key
        // change (page / sort / search / filters) to prevent table flashing.
        // Must NOT keep them across a change of resource type.
        placeholderData: (prev, prevQuery) =>
          prevQuery?.queryKey[0] === `List${type}s` ? prev : undefined,
      },
    ).data ?? [];

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

  console.log(newProps)

  return (
    <Stack {...stackProps}>
      <Group justify="space-between" w="100%">
        <Group w={{ base: "100%", xs: "fit-content" }}>
          {(isAdmin || !disableNonAdminCreate) && <RC.New {...newProps} />}
          <RC.BatchExecutions />
          <ListPagination
            page={page}
            setPage={setPage}
            count={resources.length}
          />
        </Group>

        <Group w={{ base: "100%", xs: "fit-content" }}>
          {showAvailableUpdatesQuery &&
            (type === "Stack" || type === "Deployment") && (
              <LabelledSwitch
                label="Pending Update"
                checked={filterUpdateAvailable}
                onCheckedChange={toggleFilterUpdateAvailable}
                opacity={0.7}
                fz="sm"
              />
            )}
          {showTemplateQuery && <TemplateQuerySelector />}
          {showTagsQuery && <TagsFilter />}
          <SearchInput value={search} onSearch={setSearch} />
        </Group>
      </Group>

      {Table}
    </Stack>
  );
}
