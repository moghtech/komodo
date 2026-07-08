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
import { useMemo, useState } from "react";
import { Link } from "react-router-dom";
import DashboardNoResources from "./no-resources";
import { ShowHideButton } from "mogh_ui";
import { SearchInput } from "mogh_ui";

export default function DashboardTables() {
  const [search, setSearch] = useState("");
  return (
    <Stack gap="xl">
      <Group justify="end">
        <TagsFilter />
        <SearchInput value={search} onSearch={setSearch} />
      </Group>

      <DashboardNoResources />

      {Object.entries(ResourceComponents).map(([type, RC]) => (
        <TableSection
          key={type}
          type={type as UsableResource}
          RC={RC}
          search={search}
        />
      ))}
    </Stack>
  );
}

function TableSection({
  type,
  RC,
  search,
}: {
  type: UsableResource;
  RC: RequiredResourceComponents;
  search?: string;
}) {
  const terms = useMemo(
    () =>
      search
        ?.toLowerCase()
        .split(" ")
        .map((term) => term.trim())
        .filter((term) => term),
    [search],
  );

  const tags = useTagsFilter();
  const [templates] = useTemplatesQueryBehavior();
  const _resources =
    useRead(`List${type}s`, { query: { terms, tags, templates } }).data ?? [];
  // Prevent flashing when typing / fetching
  const resources = useDebounce(_resources, 100);

  const [show, setShow] = useState(true);

  if (!resources.length) return;

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
      actions={<ShowHideButton show={show} setShow={setShow} />}
    >
      {show && <RC.Table resources={resources} />}
    </Section>
  );
}
