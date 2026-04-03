import { fmtVersion } from "@/lib/formatting";
import { useRead } from "@/lib/hooks";
import { useWebsocketMessages } from "@/lib/socket";
import { updateLogToHtml, versionIsNone } from "@/lib/utils";
import { ResourceComponents, UsableResource } from "@/resources";
import { ActionIcon, Code, Drawer, Group, Stack, Text } from "@mantine/core";
import UserAvatar from "@/components/user-avatar";
import { ICONS } from "@/lib/icons";
import { Clock, Link2, SquarePen } from "lucide-react";
import {
  CopyButton,
  fmtDateWithMinutes,
  fmtDuration,
  fmtUpperCamelcase,
} from "mogh_ui";
import { Section } from "mogh_ui";
import { MonacoDiffEditor } from "@/components/monaco";
import LoadingScreen from "@/ui/loading-screen";
import { atom, useAtom } from "jotai";
import ResourceLink from "@/resources/link";
import { To, useLocation, useNavigate } from "react-router-dom";

const updateDetailsAtom = atom<string>();

/** There is one update details modal mounted, just change the target update id */
export function useUpdateDetails() {
  const [updateId, setUpdateId] = useAtom(updateDetailsAtom);
  return {
    updateId,
    open: (updateId: string) => setUpdateId(updateId),
    close: () => setUpdateId(undefined),
  };
}

export default function UpdateDetails({
  updateId: __updateId,
}: {
  updateId?: string;
}) {
  const { updateId: _updateId, close } = useUpdateDetails();
  const updateId = __updateId ?? _updateId;
  // https://github.com/remix-run/react-router/discussions/9788#discussioncomment-4604278
  const navTo = (useLocation().key === "default" ? "/" : -1) as To;
  const nav = useNavigate();
  return (
    <Drawer
      opened={!!updateId}
      onClose={__updateId ? () => nav(navTo) : close}
      styles={{
        content: {
          flex: "none",
          width: 1400,
          maxWidth: "calc(100vw - 2rem)",
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
  const { close } = useUpdateDetails();
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
      <Group justify="space-between">
        <Text fz="h2">
          {fmtUpperCamelcase(update.operation)}{" "}
          {!versionIsNone(update.version) && fmtVersion(update.version)}
        </Text>
        <ActionIcon size="lg" variant="filled" color="red" onClick={close}>
          <ICONS.Clear size="1.3rem" />
        </ActionIcon>
      </Group>

      {/** DETAILS */}
      <Stack gap="sm">
        <UserAvatar userId={update.operator} iconSize="1.3rem" fz="md" />

        {/** RESOURCE / VERSION */}
        <Group>
          {Components ? (
            <ResourceLink
              type={update.target.type as UsableResource}
              id={update.target.id}
              onClick={close}
              fz="md"
            />
          ) : (
            <Group gap="xs" wrap="nowrap">
              <ICONS.Settings size="1rem" />
              System
            </Group>
          )}

          {update.version && (
            <Group gap="xs" wrap="nowrap">
              <ICONS.Version size="1rem" />
              {fmtVersion(update.version)}
            </Group>
          )}
        </Group>

        {/** DATE / DURATION / COPY LINK */}
        <Group>
          <Group gap="xs" wrap="nowrap">
            <ICONS.Calendar size="1rem" />
            {fmtDateWithMinutes(new Date(update.start_ts))}
          </Group>
          <Group gap="xs" wrap="nowrap">
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
              <Stack className="bordered-light" bdrs="md" p="md">
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
              <Stack className="bordered-light" bdrs="md" p="md">
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
              <Stack className="bordered-light" bdrs="md" p="md">
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
