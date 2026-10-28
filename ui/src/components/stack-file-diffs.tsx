import { useRead } from "@/lib/hooks";
import { Badge, Group, Stack, Text } from "@mantine/core";
import { Types } from "komodo_client";
import { languageFromPath, MonacoDiffEditor, ShowHideButton } from "mogh_ui";
import { useState } from "react";

export type StackFileDiffStatus = "changed" | "new" | "same";

export interface StackFileDiff {
  path: string;
  status: StackFileDiffStatus;
  /** Contents recorded at the last deploy. Undefined when the file is new. */
  deployed?: string;
  /** Contents from Core's last refresh of the Stack. */
  remote: Types.StackRemoteFileContents;
}

/**
 * Compares each tracked file from Core's last refresh with the copy recorded
 * at the last deploy. This is the comparison the sync uses to decide
 * "File contents for <path> have changed".
 */
export function stackFileDiffs(
  info: Types.StackInfo | undefined,
): StackFileDiff[] {
  const remote = info?.remote_contents;
  if (!remote) return [];
  const deployed = info?.deployed_contents;
  return remote.map((file) => {
    const match = deployed?.find((d) => d.path === file.path);
    if (!match) return { path: file.path, status: "new", remote: file };
    return {
      path: file.path,
      status: match.contents === file.contents ? "same" : "changed",
      deployed: match.contents,
      remote: file,
    };
  });
}

export function StackFileDiffBadge({
  status,
}: {
  status: StackFileDiffStatus;
}) {
  switch (status) {
    case "changed":
      return (
        <Badge color="yellow" variant="light">
          differs from deployed
        </Badge>
      );
    case "new":
      return (
        <Badge color="blue" variant="light">
          not deployed yet
        </Badge>
      );
    case "same":
      return (
        <Badge color="gray" variant="light">
          same as deployed
        </Badge>
      );
  }
}

/** What Core recorded for the file: the services it is attached to and what a change requires. */
export function StackFileRequirement({
  file,
}: {
  file: Types.StackRemoteFileContents;
}) {
  const requires = file.requires ?? Types.StackFileRequires.None;
  const services = file.services ?? [];
  if (requires === Types.StackFileRequires.None && services.length === 0) {
    return null;
  }
  return (
    <Text c="dimmed" fz="sm">
      {requires === Types.StackFileRequires.None
        ? "no action"
        : `requires ${requires}`}
      {services.length > 0 ? ` for ${services.join(", ")}` : ""}
    </Text>
  );
}

export function StackFileDiffView({
  diff,
  defaultOpen = true,
}: {
  diff: StackFileDiff;
  defaultOpen?: boolean;
}) {
  const [show, setShow] = useState(defaultOpen);
  return (
    <Stack className="bordered-light" bdrs="sm" p="md" gap="xs" w="100%">
      <Group justify="space-between">
        <Group ff="monospace" gap="xs">
          <Text c="dimmed">File:</Text>
          <Text>{diff.path}</Text>
          <StackFileDiffBadge status={diff.status} />
          <StackFileRequirement file={diff.remote} />
        </Group>
        <ShowHideButton show={show} setShow={() => setShow((show) => !show)} />
      </Group>
      {show && (
        <MonacoDiffEditor
          original={diff.deployed ?? ""}
          modified={diff.remote.contents}
          language={languageFromPath(diff.path)}
          readOnly
        />
      )}
    </Stack>
  );
}

/**
 * For a Stack the sync is about to deploy: every tracked file whose latest
 * copy differs from the deployed one, displayed as a diff.
 */
export function StackPendingFileDiffs({ stack }: { stack: string }) {
  const { data, isLoading, isError } = useRead(
    "GetStack",
    { stack },
    { refetchInterval: 30_000 },
  );
  if (isLoading) {
    return (
      <Text c="dimmed" fz="sm" w="100%">
        Loading tracked files...
      </Text>
    );
  }
  if (isError || !data?.info?.remote_contents) return null;
  const diffs = stackFileDiffs(data.info).filter(
    (diff) => diff.status !== "same",
  );
  if (diffs.length === 0) {
    return (
      <Text c="dimmed" fz="sm" w="100%">
        Every tracked file matches the deployed version.
      </Text>
    );
  }
  return (
    <Stack w="100%" gap="xs">
      <Text c="dimmed" fz="sm">
        {diffs.length} tracked file{diffs.length === 1 ? "" : "s"} differ from
        the deployed cop{diffs.length === 1 ? "y" : "ies"}, as of Core's last
        refresh of this Stack (deployed {data.info.deployed_hash ?? "unknown"},
        latest {data.info.latest_hash ?? "unknown"}).
      </Text>
      {diffs.map((diff) => (
        <StackFileDiffView
          key={diff.path}
          diff={diff}
          defaultOpen={diffs.length === 1}
        />
      ))}
    </Stack>
  );
}
