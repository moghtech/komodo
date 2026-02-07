import { filterMultitermBySplit } from "@/lib/utils";
import { ICONS } from "@/theme/icons";
import Section, { SectionProps } from "@/ui/section";
import { Box, GroupProps, TextInput } from "@mantine/core";
import { useMemo, useState } from "react";
import DockerOptions from "./options";

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
          <TextInput
            placeholder="search..."
            leftSection={<ICONS.Search size="1rem" />}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </Box>
      }
      {...props}
    >
      <DockerOptions options={filtered} {...groupProps} />
    </Section>
  );
}
