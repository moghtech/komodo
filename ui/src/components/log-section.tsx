import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import LogViewer from "@/ui/log-viewer";
import { Section, SectionProps } from "mogh_ui";
import {
  ActionIcon,
  Button,
  Group,
  SegmentedControl,
  Select,
  Switch,
  Text,
  TextInput,
} from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { notifications } from "@mantine/notifications";
import { Types } from "komodo_client";
import { ReactNode, useState } from "react";

export type LogTarget =
  | { type: "Container"; serverId: string; container: string }
  | { type: "SwarmService"; swarmId: string; service: string }
  | { type: "Deployment"; deploymentId: string }
  | { type: "Stack"; stackId: string; services: string[] };

export type LogStream = "stdout" | "stderr";

export interface LogSectionProps extends Omit<SectionProps, "children"> {
  target: LogTarget;
  extraController?: ReactNode;
  disabled?: boolean;
}

export default function LogSection({ disabled, ...props }: LogSectionProps) {
  if (disabled) {
    return (
      <Section {...props}>
        <Text>Logs are not available</Text>
      </Section>
    );
  }
  return <LogSectionInner {...props} />;
}

export function LogSectionInner({
  target,
  extraController,
  ...props
}: LogSectionProps) {
  const [timestamps, setTimestamps] = useLocalStorage({
    key: "log-timestamps-v1",
    defaultValue: false,
  });
  const [poll, setPoll] = useLocalStorage({
    key: "log-poll-v1",
    defaultValue: false,
  });
  const [_stream, setStream] = useState<LogStream>("stdout");
  const [tail, setTail] = useState("100");
  const [terms, setTerms] = useState<string[]>([]);
  const [invert, setInvert] = useState(false);
  const [search, setSearch] = useState("");

  const {
    data: standardLog,
    refetch: standardRefetch,
    isFetching: standardLoading,
  } = useRead(
    `Get${target.type}Log`,
    {
      ...targetParams(target),
      timestamps,
      tail: Number(tail),
    } as any,
    { enabled: !terms.length, refetchInterval: poll ? 10_000 : undefined },
  );

  const {
    data: searchLog,
    refetch: searchRefetch,
    isFetching: searchLoading,
  } = useRead(
    `Search${target.type}Log`,
    {
      ...targetParams(target),
      terms,
      combinator: Types.SearchCombinator.And,
      invert,
      timestamps,
    } as any,
    { enabled: !!terms.length, refetchInterval: poll ? 10_000 : undefined },
  );

  const [log, refetch, loading] = terms.length
    ? [searchLog, searchRefetch, searchLoading]
    : [standardLog, standardRefetch, standardLoading];

  const addTerm = () => {
    if (!search.length) return;
    if (terms.includes(search)) {
      notifications.show({ message: "Search term is already present" });
      setSearch("");
      return;
    }
    setTerms([...terms, search]);
    setSearch("");
  };

  const clearSearch = () => {
    setSearch("");
    setTerms([]);
  };

  // Convenience select stderr first / disabled selector if its only one with logs
  const hasStdout = !!log?.stdout;
  const hasStderr = !!log?.stderr;
  const streamSelectDisabled = !hasStdout || !hasStderr;
  const stream =
    _stream === "stdout" && !hasStdout && hasStderr
      ? "stderr"
      : _stream === "stderr" && !hasStderr && hasStdout
        ? "stdout"
        : _stream;

  return (
    <Section
      actions={
        <Group gap="lg">
          <Switch
            label="Invert"
            checked={invert}
            onChange={(e) => setInvert(e.target.checked)}
          />

          {terms.map((term, index) => (
            <Button
              key={term}
              variant="filled"
              color="red"
              rightSection={<ICONS.Remove size="1rem" />}
              onClick={() => setTerms(terms.filter((_, i) => i !== index))}
            >
              {term}
            </Button>
          ))}

          <TextInput
            placeholder="Search Logs"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            onBlur={addTerm}
            onKeyDown={(e) => {
              if (e.key === "Enter") addTerm();
            }}
            w={{ base: 180, xl: 240 }}
            leftSection={<ICONS.Search size="1rem" />}
            rightSection={
              <ActionIcon
                onClick={clearSearch}
                color="red"
                disabled={!terms.length}
              >
                <ICONS.Delete size="1rem" />
              </ActionIcon>
            }
          />

          <SegmentedControl
            value={stream}
            onChange={(stream) => setStream(stream as LogStream)}
            data={["stdout", "stderr"]}
            disabled={streamSelectDisabled}
          />

          <ActionIcon size="lg" onClick={() => refetch()} loading={loading}>
            <ICONS.Refresh size="1rem" />
          </ActionIcon>

          <Switch
            label="Timestamps"
            checked={timestamps}
            onChange={(e) => setTimestamps(e.target.checked)}
          />

          <Switch
            label="Poll"
            checked={poll}
            onChange={(e) => setPoll(e.target.checked)}
          />

          <Select
            value={tail}
            onChange={(tail) => tail && setTail(tail)}
            data={["100", "500", "1000", "5000"].map((value) => ({
              value,
              label: `${value} lines`,
            }))}
            disabled={search.length > 0}
            w={130}
          />

          {extraController}
        </Group>
      }
      {...props}
    >
      <LogViewer log={log?.[stream]} />
    </Section>
  );
}

function targetParams(target: LogTarget) {
  switch (target.type) {
    case "Container":
      return {
        server: target.serverId,
        container: target.container,
      };
    case "SwarmService":
      return {
        swarm: target.swarmId,
        service: target.service,
      };
    case "Deployment":
      return {
        deployment: target.deploymentId,
      };
    case "Stack":
      return {
        stack: target.stackId,
        services: target.services,
      };
  }
}
