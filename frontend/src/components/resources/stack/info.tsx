import { Section } from "@components/layouts";
import { ReactNode, useState } from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@ui/card";
import { useFullStack, useStack } from ".";
import { cn, updateLogToHtml } from "@lib/utils";
import { language_from_path, MonacoEditor } from "@components/monaco";
import { usePermissions } from "@lib/hooks";
import { ConfirmUpdate } from "@components/config/util";
import { useLocalStorage, useWrite } from "@lib/hooks";
import { Button } from "@ui/button";
import { FilePlus, History } from "lucide-react";
import { useToast } from "@ui/use-toast";
import { ConfirmButton, ShowHideButton, CopyButton } from "@components/util";
import { DEFAULT_STACK_FILE_CONTENTS } from "./config";
import { Types } from "komodo_client";

export const StackInfo = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) => {
  const [edits, setEdits] = useLocalStorage<Record<string, string | undefined>>(
    `stack-${id}-edits`,
    {}
  );
  const [show, setShow] = useState<Record<string, boolean | undefined>>({});
  const { canWrite } = usePermissions({ type: "Stack", id });
  const { toast } = useToast();
  const { mutateAsync, isPending } = useWrite("WriteStackFileContents", {
    onSuccess: (res) => {
      toast({
        title: res.success ? "Contents written." : "Failed to write contents.",
        variant: res.success ? undefined : "destructive",
      });
    },
  });

  const not_down = useStack(id)?.info.state !== Types.StackState.Down;
  const stack = useFullStack(id);
  // const state = useStack(id)?.info.state ?? Types.StackState.Unknown;
  // const is_down = [Types.StackState.Down, Types.StackState.Unknown].includes(
  //   state
  // );

  const file_on_host = stack?.config?.files_on_host ?? false;
  const git_repo = !!(stack?.config?.repo || stack?.config?.linked_repo);
  const canEdit = canWrite && (file_on_host || git_repo);
  const editFileCallback = (path: string) => (contents: string) =>
    setEdits({ ...edits, [path]: contents });

  // Collect deployed / latest contents, joining
  // them by path.
  // Only unmatched latest contents end up in latest_contents.
  // const deployed_contents: {
  //   path: string;
  //   deployed: string;
  //   modified: string | undefined;
  // }[] = [];

  // if (!is_down) {
  //   for (const content of stack?.info?.deployed_contents ?? []) {
  //     const latest = stack?.info?.remote_contents?.find(
  //       (latest) => latest.path === content.path
  //     );
  //     const modified =
  //       latest?.contents &&
  //       (latest.contents !== content.contents ? latest.contents : undefined);
  //     deployed_contents.push({
  //       path: content.path,
  //       deployed: content.contents,
  //       modified,
  //     });
  //   }
  // }

  const latest_contents = stack?.info?.remote_contents;
  const latest_errors = stack?.info?.remote_errors;

  // Contents will be default hidden if there is more than 2 file editor to show
  const default_show_contents = !latest_contents || latest_contents.length < 3;

  return (
    <Section titleOther={titleOther}>
      {/* Errors */}
      {latest_errors &&
        latest_errors.length > 0 &&
        latest_errors.map((error) => (
          <Card key={error.path} className="flex flex-col gap-4">
            <CardHeader className="flex flex-row justify-between items-center pb-0">
              <div className="font-mono flex gap-2">
                <div className="text-muted-foreground">Path:</div>
                {error.path}
              </div>
              {canEdit && (
                <ConfirmButton
                  title="Initialize File"
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
                  loading={isPending}
                />
              )}
            </CardHeader>
            <CardContent className="pr-8">
              <pre
                dangerouslySetInnerHTML={{
                  __html: updateLogToHtml(error.contents),
                }}
                className="max-h-[500px] overflow-y-auto"
              />
            </CardContent>
          </Card>
        ))}

      {/* Update deployed contents with diff */}
      {/* {!is_down && deployed_contents.length > 0 && (
        <Card>
          <CardHeader className="flex flex-col gap-2">
            deployed contents:{" "}
          </CardHeader>
          <CardContent>
            {deployed_contents.map((content) => {
              return (
                <pre key={content.path} className="flex flex-col gap-2">
                  <div className="flex justify-between items-center">
                    <div>path: {content.path}</div>
                    {canEdit && (
                      <div className="flex items-center gap-2">
                        <Button
                          variant="outline"
                          onClick={() =>
                            setEdits({ ...edits, [content.path]: undefined })
                          }
                          className="flex items-center gap-2"
                          disabled={!edits[content.path]}
                        >
                          <History className="w-4 h-4" />
                          Reset
                        </Button>
                        <ConfirmUpdate
                          previous={{
                            contents: content.modified ?? content.deployed,
                          }}
                          content={{ contents: edits[content.path] }}
                          onConfirm={() => {
                            if (stack) {
                              mutateAsync({
                                stack: stack.name,
                                file_path: content.path,
                                contents: edits[content.path]!,
                              }).then(() =>
                                setEdits({
                                  ...edits,
                                  [content.path]: undefined,
                                })
                              );
                            }
                          }}
                          disabled={!edits[content.path]}
                        />
                      </div>
                    )}
                  </div>
                  {content.modified ? (
                    <MonacoDiffEditor
                      original={"# Deployed contents\n" + content.deployed}
                      modified={edits[content.path] ?? content.modified}
                      language="yaml"
                      readOnly={!canEdit}
                      hideUnchangedRegions={false}
                      onModifiedValueChange={editFileCallback(content.path)}
                    />
                  ) : (
                    <MonacoEditor
                      value={edits[content.path] ?? content.deployed}
                      language="yaml"
                      readOnly={!canEdit}
                      onValueChange={editFileCallback(content.path)}
                    />
                  )}
                </pre>
              );
            })}
          </CardContent>
        </Card>
      )} */}

      {/* Update latest contents */}
      {latest_contents &&
        latest_contents.length > 0 &&
        latest_contents.map((content) => {
          const showContents = show[content.path] ?? default_show_contents;
          const handleToggleShow = () => {
            setShow((show) => ({
              ...show,
              [content.path]: !show[content.path],
            }));
          };
          return (
            <Card key={content.path} className="flex flex-col gap-4">
              <CardHeader
                className={cn(
                  "flex flex-row justify-between items-center group cursor-pointer",
                  showContents && "pb-0"
                )}
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
                <CardTitle className="font-mono flex gap-2 items-center">
                  <div className="flex gap-2 items-center">
                    <span className="text-muted-foreground">File:</span>
                    <span>{content.path}</span>
                    <span onClick={(e) => e.stopPropagation()} data-copy-button>
                      <CopyButton content={content.path} label="file path" />
                    </span>
                  </div>
                </CardTitle>
                <div className="flex items-center gap-2">
                  {canEdit && (
                    <>
                      <Button
                        variant="outline"
                        onClick={(e) => {
                          e.stopPropagation();
                          setEdits({ ...edits, [content.path]: undefined });
                        }}
                        className="flex items-center gap-2"
                        disabled={!edits[content.path]}
                      >
                        <History className="w-4 h-4" />
                        Reset
                      </Button>
                      <span onClick={(e) => e.stopPropagation()}>
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
                                })
                              );
                            }
                          }}
                          disabled={!edits[content.path]}
                          language="yaml"
                          loading={isPending}
                        />
                      </span>
                    </>
                  )}
                  <ShowHideButton
                    show={showContents}
                    setShow={(val) => setShow({ ...show, [content.path]: val })}
                  />
                </div>
              </CardHeader>
              {showContents && (
                <CardContent className="pr-8">
                  <MonacoEditor
                    value={edits[content.path] ?? content.contents}
                    language={language_from_path(content.path)}
                    readOnly={!canEdit}
                    filename={content.path}
                    onValueChange={editFileCallback(content.path)}
                  />
                </CardContent>
              )}
            </Card>
          );
        })}

      {stack?.info?.deployed_config && not_down && (
        <Card className="flex flex-col gap-4">
          <CardHeader className="pb-0">
            <CardTitle className="font-mono">Deployed config:</CardTitle>
            <CardDescription>
              Output of '<code>docker compose config</code>' when Stack was last
              deployed.
            </CardDescription>
          </CardHeader>

          <CardContent className="pr-8">
            <MonacoEditor
              value={stack.info.deployed_config}
              language="yaml"
              readOnly
            />
          </CardContent>
        </Card>
      )}
    </Section>
  );
};
