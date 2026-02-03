import { usePermissions, useWrite } from "@/lib/hooks";
import { useLocalStorage } from "@mantine/hooks";
import { notifications } from "@mantine/notifications";
import { ReactNode, useState } from "react";
import { DEFAULT_STACK_FILE_CONTENTS, StackComponents } from ".";
import { Types } from "komodo_client";
import Section from "@/ui/section";
import { Button, Code, Group, Stack, Text } from "@mantine/core";
import ConfirmButton from "@/ui/confirm-button";
import { FilePlus } from "lucide-react";
import { updateLogToHtml } from "@/lib/utils";
import CopyButton from "@/ui/copy-button";
import { ICONS } from "@/lib/icons";
import ConfirmUpdate from "@/ui/config/confirm";
import ShowHideButton from "@/ui/show-hide-button";
import { languageFromPath, MonacoEditor } from "@/components/monaco";

export default function StackInfo({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const [edits, setEdits] = useLocalStorage<Record<string, string | undefined>>(
    { key: `stack-${id}-edits`, defaultValue: {} },
  );
  const [show, setShow] = useState<Record<string, boolean | undefined>>({});
  const { canWrite } = usePermissions({ type: "Stack", id });

  const { mutateAsync, isPending } = useWrite("WriteStackFileContents", {
    onSuccess: (res) => {
      notifications.show({
        message: res.success
          ? "Contents written."
          : "Failed to write contents.",
        color: res.success ? undefined : "red",
      });
    },
  });

  const notDown =
    StackComponents.useListItem(id)?.info.state !== Types.StackState.Down;
  const stack = StackComponents.useFull(id);

  const filesOnHost = stack?.config?.files_on_host ?? false;
  const gitRepo = !!(stack?.config?.repo || stack?.config?.linked_repo);
  const canEdit = canWrite && (filesOnHost || gitRepo);
  const editFileCallback = (path: string) => (contents: string) =>
    setEdits({ ...edits, [path]: contents });

  const latestContents = stack?.info?.remote_contents;
  const latestErrors = stack?.info?.remote_errors;

  // Contents will be default hidden if there is more than 2 file editor to show
  const defaultShowContents = !latestContents || latestContents.length < 3;

  return (
    <Section gap="xl" titleOther={titleOther}>
      {/* Errors */}
      {latestErrors &&
        latestErrors.length > 0 &&
        latestErrors.map((error) => (
          <Stack
            bd="1px solid var(--mantine-color-accent-border-0)"
            bdrs="md"
            p="xl"
          >
            {/* HEADER */}
            <Group justify="between">
              <Group ff="monospace">
                <Text c="dimmed">Path:</Text>
                {error.path}
              </Group>

              {canEdit && (
                <ConfirmButton
                  loading={isPending}
                  icon={<FilePlus className="w-4 h-4" />}
                  onClick={() => {
                    if (stack) {
                      mutateAsync({
                        stack: stack.name,
                        file_path: error.path,
                        contents: DEFAULT_STACK_FILE_CONTENTS,
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
                __html: updateLogToHtml(error.contents),
              }}
              style={{ overflowY: "auto" }}
            />
          </Stack>
        ))}

      {/* Update latest contents */}
      {latestContents &&
        latestContents.length > 0 &&
        latestContents.map((content) => {
          const showContents = show[content.path] ?? defaultShowContents;
          const handleToggleShow = () => {
            setShow((show) => ({
              ...show,
              [content.path]: !(show[content.path] ?? defaultShowContents),
            }));
          };
          return (
            <Stack
              bd="1px solid var(--mantine-color-accent-border-0)"
              bdrs="md"
              p="xl"
            >
              {/* HEADER */}
              <Group
                justify="space-between"
                style={{ cursor: "pointer" }}
                onClick={handleToggleShow}
                tabIndex={0}
                role="button"
                aria-pressed={showContents}
                onKeyDown={(e) => {
                  if (
                    (e.key === "Enter" || e.key === " ") &&
                    e.target === e.currentTarget
                  ) {
                    if (e.key === " ") e.preventDefault();
                    handleToggleShow();
                  }
                }}
              >
                {/* PATH */}
                <Group ff="monospace">
                  <Text c="dimmed">File:</Text>
                  {content.path}
                  <CopyButton label="file path" content={content.path} />
                </Group>

                {/* SAVE */}
                <Group>
                  {canEdit && (
                    <>
                      <Button
                        variant="outline"
                        leftSection={<ICONS.History size="1rem" />}
                        disabled={!edits[content.path]}
                        onClick={(e) => {
                          e.stopPropagation();
                          setEdits({ ...edits, [content.path]: undefined });
                        }}
                      >
                        Reset
                      </Button>
                      <ConfirmUpdate
                        previous={{ contents: content.contents }}
                        content={{ contents: edits[content.path] }}
                        onConfirm={async () => {
                          if (stack) {
                            return await mutateAsync({
                              stack: stack.name,
                              file_path: content.path,
                              contents: edits[content.path]!,
                            }).then(() =>
                              setEdits({
                                ...edits,
                                [content.path]: undefined,
                              }),
                            );
                          }
                        }}
                        disabled={!edits[content.path]}
                        language="yaml"
                        loading={isPending}
                      />
                    </>
                  )}
                  {/* The toggle onClick is given to entire header */}
                  <ShowHideButton show={showContents} setShow={() => {}} />
                </Group>
              </Group>

              {/* CONTENTS */}
              {showContents && (
                <MonacoEditor
                  value={edits[content.path] ?? content.contents}
                  language={languageFromPath(content.path)}
                  readOnly={!canEdit}
                  filename={content.path}
                  onValueChange={editFileCallback(content.path)}
                />
              )}
            </Stack>
          );
        })}

      {stack?.info?.deployed_config && notDown && (
        <Stack
          bd="1px solid var(--mantine-color-accent-border-0)"
          bdrs="md"
          p="xl"
        >
          {/* HEADER */}
          <Stack ff="monospace" gap="0">
            <Text fz="md">Deployed config:</Text>
            <Text fz="sm" c="dimmed">
              Output of '<code>docker compose config</code>' when Stack was last
              deployed.
            </Text>
          </Stack>

          <MonacoEditor
            value={stack.info.deployed_config}
            language="yaml"
            readOnly
          />
        </Stack>
      )}
    </Section>
  );
}
