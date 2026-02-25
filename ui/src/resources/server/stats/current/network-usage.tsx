import { fmtRateBytes } from "@/lib/formatting";
import InfoCard from "@/ui/info-card";
import { Group, Stack, Text } from "@mantine/core";
import { Types } from "komodo_client";

export default function ServerNetworkUsage({
  stats,
}: {
  stats: Types.SystemStats | undefined;
}) {
  return (
    <InfoCard title="Network Usage" w={{ base: "100%", lg: 300 }} gap="xs">
      <Stack gap="0">
        <Group justify="space-between">
          <Text>Ingress</Text>
          <Text>{fmtRateBytes(stats?.network_ingress_bytes ?? 0)}</Text>
        </Group>
        <Group justify="space-between">
          <Text>Egress</Text>
          <Text>{fmtRateBytes(stats?.network_egress_bytes ?? 0)}</Text>
        </Group>
      </Stack>
    </InfoCard>
  );
}
