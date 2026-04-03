import { hexColorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { getUpdateQuery, updateLogToHtml } from "@/lib/utils";
import { Section } from "mogh_ui";
import { Box, Code, Group, Stack, Text } from "@mantine/core";

export default function ActionLastRun({ id }: { id: string }) {
  const update = useRead("ListUpdates", {
    query: {
      ...getUpdateQuery({ type: "Action", id }, undefined),
      operation: "RunAction",
    },
  }).data?.updates[0];

  const full_update = useRead(
    "GetUpdate",
    { id: update?.id! },
    { enabled: !!update?.id },
  ).data;

  const log = full_update?.logs.find((log) => log.stage === "Execute Action");

  if (!log?.stdout && !log?.stderr) {
    return (
      <Section withBorder>
        <Text c={hexColorByIntention("Neutral")}>Never run</Text>
      </Section>
    );
  }

  return (
    <Stack>
      {!log?.stdout && !log?.stderr && (
        <Box className="bordered-light" p="lg" bdrs="md">
          <Text c={hexColorByIntention("Neutral")}>Never run</Text>
        </Box>
      )}
      {log.stdout && (
        <Stack className="bordered-light" p="lg" bdrs="md">
          <Group gap="xs">
            <Text>Last run:</Text>
            <Text c={hexColorByIntention("Good")}>Stdout</Text>
          </Group>
          <Code
            component="pre"
            fz="sm"
            mah={500}
            dangerouslySetInnerHTML={{
              __html: updateLogToHtml(log.stdout),
            }}
            style={{ overflowY: "auto" }}
          />
        </Stack>
      )}
      {log.stderr && (
        <Stack className="bordered-light" p="lg" bdrs="md">
          <Group gap="xs">
            <Text>Last run:</Text>
            <Text c={hexColorByIntention("Critical")}>Stderr</Text>
          </Group>
          <Code
            component="pre"
            fz="sm"
            mah={500}
            dangerouslySetInnerHTML={{
              __html: updateLogToHtml(log.stderr),
            }}
            style={{ overflowY: "auto" }}
          />
        </Stack>
      )}
    </Stack>
  );
}
