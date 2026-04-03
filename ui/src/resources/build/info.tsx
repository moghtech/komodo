import { usePermissions, useRead, useWrite } from "@/lib/hooks";
import { useLocalStorage } from "@mantine/hooks";
import { notifications } from "@mantine/notifications";
import { ReactNode, useState } from "react";
import { useFullBuild } from ".";
import { Section } from "mogh_ui";
import { Button, Code, Group, Stack, Text } from "@mantine/core";
import ConfirmButton from "@/ui/confirm-button";
import { Clock, FilePlus } from "lucide-react";
import { DEFAULT_BUILD_DOCKERFILE_CONTENTS } from "./config";
import { updateLogToHtml } from "@/lib/utils";
import { CopyButton } from "mogh_ui";
import { ICONS } from "@/lib/icons";
import ConfirmUpdate from "@/ui/config/confirm";
import ShowHideButton from "@/ui/show-hide-button";
import { MonacoEditor } from "@/components/monaco";
import { fmtDuration } from "@/lib/formatting";

export default function BuildInfo({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const [edits, setEdits] = useLocalStorage<{ contents: string | undefined }>({
    key: `build-${id}-edits`,
    defaultValue: { contents: undefined },
  });
  const [show, setShow] = useState(true);
  const { canWrite } = usePermissions({ type: "Build", id });
  const { mutateAsync, isPending } = useWrite("WriteBuildFileContents", {
    onSuccess: (res) => {
      notifications.show({
        message: res.success
          ? "Contents written."
          : "Failed to write contents.",
        color: res.success ? "green" : "red",
      });
    },
  });

  const build = useFullBuild(id);

  const recentBuilds = useRead("ListUpdates", {
    query: { "target.type": "Build", "target.id": id, operation: "RunBuild" },
  }).data;
  const _lastBuild = recentBuilds?.updates[0];
  const lastBuild = useRead(
    "GetUpdate",
    {
      id: _lastBuild?.id!,
    },
    { enabled: !!_lastBuild },
  ).data;

  const fileOnHost = build?.config?.files_on_host ?? false;
  const gitRepo =
    build?.config?.repo || build?.config?.linked_repo ? true : false;
  const canEdit = canWrite && (fileOnHost || gitRepo);

  const remotePath = build?.info?.remote_path;
  const remoteContents = build?.info?.remote_contents;
  const remoteError = build?.info?.remote_error;

  return (
    <Section gap="xl" titleOther={titleOther}>
      {/* Errors */}
      {remoteError && remoteError.length > 0 && (
        <Stack className="bordered-light" bdrs="md" p="xl">
          {/* HEADER */}
          <Group justify="between">
            <Group ff="monospace">
              <Text c="dimmed">Path:</Text>
              {remotePath}
            </Group>

            {canEdit && (
              <ConfirmButton
                loading={isPending}
                icon={<FilePlus size="1rem" />}
                onClick={() => {
                  if (build) {
                    mutateAsync({
                      build: build.name,
                      contents: DEFAULT_BUILD_DOCKERFILE_CONTENTS,
                    });
                  }
                }}
              >
                Initialize File
              </ConfirmButton>
            )}
          </Group>

          {/* CONTENTS */}
          <Code
            component="pre"
            fz="sm"
            mah={500}
            dangerouslySetInnerHTML={{
              __html: updateLogToHtml(remoteError),
            }}
            style={{ overflowY: "auto" }}
          />
        </Stack>
      )}

      {/* Update latest contents */}
      {remoteContents && remoteContents.length > 0 && (
        <Stack className="bordered-light" bdrs="md" p="xl">
          {/* HEADER */}
          <Group
            justify="space-between"
            style={{ cursor: "pointer" }}
            onClick={() => setShow((show) => !show)}
            tabIndex={0}
            role="button"
            aria-pressed={show}
            onKeyDown={(e) => {
              if (
                (e.key === "Enter" || e.key === " ") &&
                e.target === e.currentTarget
              ) {
                if (e.key === " ") e.preventDefault();
                setShow((show) => !show);
              }
            }}
          >
            {/* PATH */}
            {remotePath && (
              <Group ff="monospace">
                <Text c="dimmed">File:</Text>
                {remotePath}
                <CopyButton label="file path" content={remotePath} />
              </Group>
            )}

            {/* SAVE */}
            <Group>
              {canEdit && (
                <>
                  <Button
                    variant="outline"
                    leftSection={<ICONS.History size="1rem" />}
                    disabled={!edits.contents}
                    onClick={(e) => {
                      e.stopPropagation();
                      setEdits({ contents: undefined });
                    }}
                  >
                    Reset
                  </Button>
                  <ConfirmUpdate
                    original={{ contents: remoteContents }}
                    update={{ contents: edits.contents }}
                    onConfirm={async () => {
                      if (build) {
                        return await mutateAsync({
                          build: build.name,
                          contents: edits.contents!,
                        }).then(() => setEdits({ contents: undefined }));
                      }
                    }}
                    disabled={!edits.contents}
                    language="dockerfile"
                    loading={isPending}
                  />
                </>
              )}
              <ShowHideButton show={show} setShow={() => setShow((show) => !show)} />
            </Group>
          </Group>

          {/* CONTENTS */}
          {show && (
            <MonacoEditor
              value={edits.contents ?? remoteContents}
              language="dockerfile"
              readOnly={!canEdit}
              onValueChange={(contents) => setEdits({ contents })}
            />
          )}
        </Stack>
      )}

      {lastBuild && lastBuild.logs.length > 0 && (
        <Code fz="lg" fw="bold" w="fit-content">
          Last Build Logs
        </Code>
      )}
      {lastBuild &&
        lastBuild.logs?.map((log, i) => (
          <Section
            key={i}
            title={log.stage}
            titleFz="h3"
            description={
              <Group c="dimmed" gap="xs">
                <Text>
                  Stage {i + 1} of {lastBuild.logs.length}
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
    </Section>
  );
}
