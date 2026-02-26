import TagsFilter from "@/components/tags/filter";
import { useFilterResources, useRead } from "@/lib/hooks";
import { usableResourcePath } from "@/lib/utils";
import {
  RequiredResourceComponents,
  ResourceComponents,
  UsableResource,
} from "@/resources";
import { ICONS } from "@/theme/icons";
import Section from "@/ui/section";
import { Group, Stack, Text } from "@mantine/core";
import { Types } from "komodo_client";
import { useState } from "react";
import { Link } from "react-router-dom";
import DashboardNoResources from "./no-resources";
import ShowHideButton from "@/ui/show-hide-button";
import SearchInput from "@/ui/search-input";

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
  const resources = useRead(`List${type}s`, {}).data;

  const filtered = useFilterResources(
    resources as Types.ResourceListItem<unknown>[],
    search,
  );

  let count = filtered.length;

  const [show, setShow] = useState(true);

  if (!count) return;

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
      {show && <RC.Table resources={filtered} />}
    </Section>
  );
}
