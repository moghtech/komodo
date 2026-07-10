import TagsFilter from "@/components/tags/filter";
import { useRead, useTagsFilter, useTemplatesQueryBehavior } from "@/lib/hooks";
import { usableResourcePath } from "@/lib/utils";
import {
  RequiredResourceComponents,
  ResourceComponents,
  UsableResource,
} from "@/resources";
import { ICONS } from "@/lib/icons";
import { Section, useDebounce } from "mogh_ui";
import { Group, Stack, Text } from "@mantine/core";
import { useEffect, useMemo, useState } from "react";
import ListPagination from "@/components/list-pagination";
import { Link } from "react-router-dom";
import DashboardNoResources from "./no-resources";
import { ShowHideButton } from "mogh_ui";
import { SearchInput } from "mogh_ui";

export default function DashboardTables() {
  const [search, setSearch] = useState("");
  const debouncedSearch = useDebounce(search, 200);
  const terms = useMemo(() => {
    return debouncedSearch
      .toLowerCase()
      .split(" ")
      .map((term) => term.trim())
      .filter((term) => term);
  }, [debouncedSearch]);

  const Tables = useMemo(
    () =>
      Object.entries(ResourceComponents).map(([type, RC]) => (
        <TableSection
          key={type}
          type={type as UsableResource}
          RC={RC}
          terms={terms}
        />
      )),
    [terms],
  );
  return (
    <Stack gap="xl">
      <Group justify="end">
        <TagsFilter />
        <SearchInput value={search} onSearch={setSearch} />
      </Group>

      <DashboardNoResources />

      {Tables}
    </Stack>
  );
}

function TableSection({
  type,
  RC,
  terms,
}: {
  type: UsableResource;
  RC: RequiredResourceComponents;
  terms: string[];
}) {
  const [show, setShow] = useState(true);
  const tags = useTagsFilter();
  const [templates] = useTemplatesQueryBehavior();

  const [page, setPage] = useState(0);
  // Server side sort, passed up from the table.
  const [sort, setSort] = useState<{
    sort_by?: string;
    sort_desc?: boolean;
  }>({});
  // Set to page 0 whenever any filter or the sort changes,
  // otherwise the query can point past the last page and come back empty.
  useEffect(() => {
    setPage(0);
  }, [terms, tags, templates, sort.sort_by, sort.sort_desc]);

  const _resources =
    useRead(`List${type}s`, {
      query: { terms, tags, templates },
      page,
      sort_by: sort.sort_by as any,
      sort_desc: sort.sort_desc,
    }).data ?? [];
  // Prevent flashing when typing / fetching
  const resources = useDebounce(_resources, 300);

  const Table = useMemo(
    () => show && <RC.Table resources={resources} onServerSort={setSort} />,
    [resources, show],
  );

  // Keep the section visible on empty pages past the first,
  // so the pagination controls remain available to navigate back.
  if (!resources.length && page === 0) return;

  const Icon = ICONS[type];

  return (
    <Section
      key={type}
      icon={<Icon size="1.3rem" />}
      titleNode={
        <Text
          fz="h2"
          renderRoot={(props) => (
            <Link to={`/${usableResourcePath(type)}`} {...props} />
          )}
        >
          {type + "s"}
        </Text>
      }
      actions={
        <Group gap="xs">
          {show && (
            <ListPagination
              page={page}
              setPage={setPage}
              count={resources.length}
            />
          )}
          <ShowHideButton show={show} setShow={setShow} />
        </Group>
      }
    >
      {Table}
    </Section>
  );
}
