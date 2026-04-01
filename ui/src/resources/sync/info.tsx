import { usePermissions, useWrite } from "@/lib/hooks";
import { useLocalStorage } from "@mantine/hooks";
import { notifications } from "@mantine/notifications";
import { ReactNode, useState } from "react";
import { useFullResourceSync } from ".";
import Section from "@/ui/section";
import { Button, Code, Group, Stack, Text } from "@mantine/core";
import ConfirmButton from "@/ui/confirm-button";
import { FilePlus } from "lucide-react";
import { updateLogToHtml } from "@/lib/utils";
import { ICONS } from "@/theme/icons";
import ConfirmUpdate from "@/ui/config/confirm";
import DividedChildren from "@/ui/divided-children";
import ShowHideButton from "@/ui/show-hide-button";
import { MonacoEditor } from "@/components/monaco";

export default function ResourceSyncInfo({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const [edits, setEdits] = useLocalStorage<Record<string, string | undefined>>(
    { key: `sync-${id}-edits-v1`, defaultValue: {} },
  );
  const [show, setShow] = useState<Record<string, boolean | undefined>>({});
  const { canWrite } = usePermissions({ type: "ResourceSync", id });

  const { mutateAsync: writeContents, isPending } = useWrite(
    "WriteSyncFileContents",
    {
      onSuccess: (res) => {
        notifications.show({
          message: res.success
            ? "Contents written."
            : "Failed to write contents.",
          color: res.success ? "green" : "red",
        });
      },
    },
  );
  const sync = useFullResourceSync(id);
  const filesOnHost = sync?.config?.files_on_host ?? false;
  const gitRepo =
    sync?.config?.repo || sync?.config?.linked_repo ? true : false;
  const canEdit = canWrite && (filesOnHost || gitRepo);
  const editFileCallback = (keyPath: string) => (contents: string) =>
    setEdits({ ...edits, [keyPath]: contents });

  const latestContents = sync?.info?.remote_contents;
  const latestErrors = sync?.info?.remote_errors;

  // Contents will be default hidden if there is more than 2 file editor to show
  const defaultShowContents = !latestContents || latestContents.length < 3;

  return (
    <Section gap="xl" titleOther={titleOther}>
      {/* Errors */}
      {latestErrors &&
        latestErrors.length > 0 &&
        latestErrors.map((error) => (
          <Stack
            className="bordered-light"
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
                  icon={<FilePlus size="1rem" />}
                  onClick={() => {
                    if (sync) {
                      writeContents({
                        sync: sync.name,
                        resource_path: error.resource_path ?? "",
                        file_path: error.path,
                        contents: "## Add resources to get started\n",
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
        latestContents.map((content) => {
          const keyPath = content.resource_path + "/" + content.path;
          const showContents = show[keyPath] ?? defaultShowContents;
          const handleToggleShow = () => {
            setShow((show) => ({
              ...show,
              [keyPath]: !(show[keyPath] ?? defaultShowContents),
            }));
          };
          return (
            <Stack
              className="bordered-light"
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
                <DividedChildren ff="monospace">
                  {content.resource_path && (
                    <Group gap="xs">
                      <Text c="dimmed">Folder:</Text>
                      {content.resource_path}
                    </Group>
                  )}
                  <Group gap="xs">
                    <Text c="dimmed">File:</Text>
                    {content.path}
                  </Group>
                </DividedChildren>

                {/* SAVE */}
                <Group>
                  {canEdit && (
                    <>
                      <Button
                        variant="outline"
                        leftSection={<ICONS.History size="1rem" />}
                        disabled={!edits[keyPath]}
                        onClick={(e) => {
                          e.stopPropagation();
                          setEdits({ ...edits, [keyPath]: undefined });
                        }}
                      >
                        Reset
                      </Button>
                      <ConfirmUpdate
                        original={{ contents: content.contents }}
                        update={{ contents: edits[keyPath] }}
                        onConfirm={async () => {
                          if (sync) {
                            return await writeContents({
                              sync: sync.name,
                              resource_path: content.resource_path ?? "",
                              file_path: content.path,
                              contents: edits[keyPath]!,
                            }).then(() =>
                              setEdits({ ...edits, [keyPath]: undefined }),
                            );
                          }
                        }}
                        disabled={!edits[keyPath]}
                        language="fancy_toml"
                        loading={isPending}
                      />
                    </>
                  )}
                  <ShowHideButton show={showContents} setShow={handleToggleShow} />
                </Group>
              </Group>

              {/* CONTENTS */}
              {showContents && (
                <MonacoEditor
                  value={edits[keyPath] ?? content.contents}
                  language="fancy_toml"
                  readOnly={!canEdit}
                  filename={content.path}
                  onValueChange={editFileCallback(keyPath)}
                />
              )}
            </Stack>
          );
        })}
    </Section>
  );
}
