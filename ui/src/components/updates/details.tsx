import {
  fmtDateWithMinutes,
  fmtDuration,
  fmtOperation,
  fmtVersion,
} from "@/lib/formatting";
import { useRead } from "@/lib/hooks";
import { useWebsocketMessages } from "@/lib/socket";
import { updateLogToHtml, versionIsNone } from "@/lib/utils";
import { ResourceComponents, UsableResource } from "@/resources";
import { Code, Drawer, Group, Stack, Text } from "@mantine/core";
import UserAvatar from "../user-avatar";
import { ResourceLink } from "@/resources/common";
import { ICONS } from "@/lib/icons";
import { Clock, Link2, SquarePen } from "lucide-react";
import CopyButton from "@/ui/copy-button";
import Section from "@/ui/section";
import { MonacoDiffEditor } from "@/components/monaco";
import LoadingScreen from "@/ui/loading-screen";
import { atom, useAtom } from "jotai";

const updateDetailsAtom = atom<string | undefined>(undefined);

/** There is one update details modal mounted, just change the target update id */
export function useUpdateDetails() {
  const [updateId, setUpdateId] = useAtom<string | undefined>(
    updateDetailsAtom,
  );
  return {
    updateId,
    open: (updateId: string) => setUpdateId(updateId),
    close: () => setUpdateId(undefined),
  };
}

export default function UpdateDetails() {
  const { updateId, close } = useUpdateDetails();
  return (
    <Drawer
      opened={!!updateId}
      onClose={close}
      styles={{
        content: {
          flex: "none",
          width: 1400,
          maxWidth: "calc(100vw - 4rem)",
          height: "fit-content",
        },
      }}
      withCloseButton={false}
    >
      {updateId && <UpdateDetailsContent id={updateId} />}
    </Drawer>
  );
}

export function UpdateDetailsContent({ id }: { id: string }) {
  const { data: update, refetch } = useRead("GetUpdate", { id });
  // Listen for updates on the update id and refetch
  useWebsocketMessages("update-details", (update) => {
    if (update.id === id) refetch();
  });

  if (!update) {
    return <LoadingScreen mt="0" h="50vh" />;
  }

  const Components =
    update.target.type === "System"
      ? null
      : ResourceComponents[update.target.type];

  return (
    <Stack gap="xl" m="md">
      {/** HEADER */}
      <Text fz="h1">
        {fmtOperation(update.operation)}{" "}
        {!versionIsNone(update.version) && fmtVersion(update.version)}
      </Text>

      {/** DETAILS */}
      <Stack gap="sm">
        <UserAvatar userId={update.operator} iconSize="1.3rem" />

        {/** RESOURCE / VERSION */}
        <Group gap="xs">
          {Components ? (
            <ResourceLink
              type={update.target.type as UsableResource}
              id={update.target.id}
              onClick={close}
            />
          ) : (
            <Group>
              <ICONS.Settings size="1rem" />
              System
            </Group>
          )}

          {update.version && (
            <Group>
              <ICONS.Version size="1rem" />
              {fmtVersion(update.version)}
            </Group>
          )}
        </Group>

        {/** DATE / DURATION / COPY LINK */}
        <Group>
          <Group>
            <ICONS.Calendar size="1rem" />
            {fmtDateWithMinutes(new Date(update.start_ts))}
          </Group>
          <Group>
            <Clock size="1rem" />
            {update.end_ts
              ? fmtDuration(update.start_ts, update.end_ts)
              : "ongoing"}
          </Group>
          <CopyButton
            content={`${location.origin}/updates/${update._id?.$oid}`}
            icon={<Link2 size="1rem" />}
            label="shareable link"
          />
        </Group>
      </Stack>

      <Stack>
        {/** CONFIG CHANGE DIFF */}
        {update.prev_toml && update.current_toml && (
          <Section
            title="Changes Made"
            titleFz="h3"
            icon={<SquarePen size="1.2rem" />}
            withBorder
          >
            <MonacoDiffEditor
              original={update.prev_toml}
              modified={update.current_toml}
              language="fancy_toml"
              readOnly
            />
          </Section>
        )}

        {/** LOGS */}
        {update?.logs.map((log, i) => (
          <Section
            key={i}
            title={log.stage}
            titleFz="h3"
            description={
              <Group c="dimmed" gap="xs">
                <Text>
                  Stage {i + 1} of {update.logs.length}
                </Text>
                <Text>|</Text>
                <Clock size="1rem" />
                {fmtDuration(log.start_ts, log.end_ts)}
              </Group>
            }
            gap="xs"
            withBorder
          >
            {log.command && (
              <Stack
                bd="1px solid var(--mantine-color-accent-border-0)"
                bdrs="md"
                p="md"
              >
                <Text fw="bold">command</Text>
                <Code
                  fz="sm"
                  mah={500}
                  style={{
                    overflowY: "auto",
                  }}
                >
                  {log.command}
                </Code>
              </Stack>
            )}
            {log.stdout && (
              <Stack
                bd="1px solid var(--mantine-color-accent-border-0)"
                bdrs="md"
                p="md"
              >
                <Text fw="bold">stdout</Text>
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
              <Stack
                bd="1px solid var(--mantine-color-accent-border-0)"
                bdrs="md"
                p="md"
              >
                <Text fw="bold">stderr</Text>
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
          </Section>
        ))}
      </Stack>
    </Stack>
  );
}
