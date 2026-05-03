import { filterMultitermBySplit } from "mogh_ui";
import { ICONS } from "@/lib/icons";
import { Section, SectionProps } from "mogh_ui";
import { Box, GroupProps } from "@mantine/core";
import { useMemo, useState } from "react";
import DockerOptions from "./options";
import { SearchInput } from "mogh_ui";

export interface DockerLabelsSectionProps extends Omit<
  SectionProps,
  "children"
> {
  labels: Record<string, string> | undefined;
  groupProps?: GroupProps;
}

export default function DockerLabelsSection({
  labels,
  groupProps,
  ...props
}: DockerLabelsSectionProps) {
  const [search, setSearch] = useState("");
  const entries = labels ? Object.entries(labels) : [];
  const filtered = useMemo(
    () => filterMultitermBySplit(entries, search, (item) => item),
    [entries, search],
  );

  if (entries.length === 0) return null;

  return (
    <Section
      title="Labels"
      icon={<ICONS.Tags size="1.3rem" />}
      titleRight={
        <Box pl="md">
          <SearchInput value={search} onSearch={setSearch} />
        </Box>
      }
     
      {...props}
    >
      <DockerOptions options={filtered} {...groupProps} />
    </Section>
  );
}
