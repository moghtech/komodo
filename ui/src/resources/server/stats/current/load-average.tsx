import { Types } from "komodo_client";
import { useRead } from "@/lib/hooks";
import { hexColorByIntention } from "mogh_ui";
import InfoCard from "@/ui/info-card";
import { Group, Progress, SimpleGrid, Stack, Text } from "@mantine/core";
import { useIsServerAvailable } from "@/resources/server/hooks";

export function ServerLoadAverage({
  id,
  stats,
}: {
  id: string;
  stats: Types.SystemStats | undefined;
}) {
  if (!stats?.load_average) return null;
  const { one = 0, five = 0, fifteen = 0 } = stats.load_average || {};
  const isServerAvailable = useIsServerAvailable(id);
  const cores = useRead(
    "GetSystemInformation",
    { server: id },
    { enabled: isServerAvailable },
  ).data?.core_count;

  const pct = (load: number) =>
    cores && cores > 0 ? Math.min((load / cores) * 100, 100) : undefined;
  const textColor = (load: number) => {
    const p = pct(load);
    if (p === undefined) return "text-muted-foreground";
    return p <= 50
      ? hexColorByIntention("Good")
      : p <= 80
        ? hexColorByIntention("Warning")
        : hexColorByIntention("Critical");
  };

  return (
    <InfoCard title="Load Average" w={{ base: "100%", lg: 300 }}>
      {/* CURRENT LOAD */}
      <Stack gap="0.4rem">
        <Group justify="space-between" align="end">
          <Text c={textColor(one)} fz="h2" fw="bold">
            {one.toFixed(2)}
          </Text>
          <Text c="dimmed">
            {cores && cores > 0
              ? `${(pct(one) ?? 0).toFixed(0)}% of ${cores} cores`
              : "N/A"}
          </Text>
        </Group>
        <Progress value={pct(one) ?? 0} color="bw" size="lg" />
      </Stack>

      {/* TIME INTERVALS */}
      <SimpleGrid cols={3}>
        {(
          [
            ["1m", one],
            ["5m", five],
            ["15m", fifteen],
          ] as const
        ).map(([label, value]) => (
          <Stack key={label} gap="0.1rem">
            <Group justify="space-between" gap="0">
              <Text c="dimmed">{label}</Text>
              <Text c={textColor(value)}>{value.toFixed(2)}</Text>
            </Group>
            <Progress value={pct(value) ?? 0} color="bw" />
          </Stack>
        ))}
      </SimpleGrid>
    </InfoCard>
  );
}
