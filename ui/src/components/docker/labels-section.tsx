import { filterMultitermBySplit } from "@/lib/utils";
import { ICONS } from "@/theme/icons";
import Section, { SectionProps } from "@/ui/section";
import { Box, Group, Text, TextInput } from "@mantine/core";
import { useMemo, useState } from "react";

export interface DockerLabelsSectionProps extends Omit<
  SectionProps,
  "children"
> {
  labels: Record<string, string> | undefined;
}

export default function DockerLabelsSection({
  labels,
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
      icon={<ICONS.Tags size="1rem" />}
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
      <Group gap="sm">
        {filtered.map(([key, value]) => (
          <Group key={key} gap="0" bdrs="sm" bg="accent" px="xs" py="0.2rem">
            <Text fz="sm" c="dimmed">
              {key}
            </Text>
            <Text fz="sm" c="dimmed">
              =
            </Text>
            <Text
              fz="sm"
              fw="bolder"
              maw={200}
              className="text-ellipsis"
              style={{ textWrap: "nowrap" }}
            >
              {value}
            </Text>
          </Group>
        ))}
      </Group>
    </Section>
  );
}
