import { Section } from "@components/layouts";
import { ReactNode, useState } from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@ui/card";
import { useFullBuild } from ".";
import { cn, updateLogToHtml } from "@lib/utils";
import { MonacoEditor } from "@components/monaco";
import { usePermissions } from "@lib/hooks";
import { ConfirmUpdate } from "@components/config/util";
import { useLocalStorage, useRead, useWrite } from "@lib/hooks";
import { Button } from "@ui/button";
import { Clock, FilePlus, History } from "lucide-react";
import { useToast } from "@ui/use-toast";
import { ConfirmButton, ShowHideButton } from "@components/util";
import { DEFAULT_BUILD_DOCKERFILE_CONTENTS } from "./config";
import { fmt_duration } from "@lib/formatting";

export const BuildInfo = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) => {
  const [edits, setEdits] = useLocalStorage<{ contents: string | undefined }>(
    `build-${id}-edits`,
    { contents: undefined }
  );
  const [showContents, setShowContents] = useState(true);
  const { canWrite } = usePermissions({ type: "Build", id });
  const { toast } = useToast();
  const { mutateAsync, isPending } = useWrite("WriteBuildFileContents", {
    onSuccess: (res) => {
      toast({
        title: res.success ? "Contents written." : "Failed to write contents.",
        variant: res.success ? undefined : "destructive",
      });
    },
  });

  const build = useFullBuild(id);

  const recent_builds = useRead("ListUpdates", {
    query: { "target.type": "Build", "target.id": id, operation: "RunBuild" },
  }).data;
  const _last_build = recent_builds?.updates[0];
  const last_build = useRead(
    "GetUpdate",
    {
      id: _last_build?.id!,
    },
    { enabled: !!_last_build }
  ).data;

  const file_on_host = build?.config?.files_on_host ?? false;
  const git_repo =
    build?.config?.repo || build?.config?.linked_repo ? true : false;
  const canEdit = canWrite && (file_on_host || git_repo);

  const remote_path = build?.info?.remote_path;
  const remote_contents = build?.info?.remote_contents;
  const remote_error = build?.info?.remote_error;

  return (
    <Section titleOther={titleOther}>
      {/* Errors */}
      {remote_error && remote_error.length > 0 && (
        <Card className="flex flex-col gap-4">
          <CardHeader className="flex flex-row justify-between items-center pb-0">
            <div className="font-mono flex gap-2">
              {remote_path && (
                <>
                  <div className="text-muted-foreground">Path:</div>
                  {remote_path}
                </>
              )}
            </div>
            {canEdit && (
              <ConfirmButton
                title="Initialize File"
                icon={<FilePlus className="w-4 h-4" />}
                onClick={() => {
                  if (build) {
                    mutateAsync({
                      build: build.name,
                      contents: DEFAULT_BUILD_DOCKERFILE_CONTENTS,
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
                __html: updateLogToHtml(remote_error),
              }}
              className="max-h-[500px] overflow-y-auto"
            />
          </CardContent>
        </Card>
      )}

      {/* Update latest contents */}
      {remote_contents && remote_contents.length > 0 && (
        <Card className="flex flex-col gap-4">
          <CardHeader
            className={cn(
              "flex flex-row justify-between items-center",
              showContents && "pb-0"
            )}
          >
            {remote_path && (
              <CardTitle className="font-mono flex gap-2">
                <div className="text-muted-foreground">Path:</div>
                {remote_path}
              </CardTitle>
            )}
            <div className="flex items-center gap-2">
              {canEdit && (
                <>
                  <Button
                    variant="outline"
                    onClick={() => setEdits({ contents: undefined })}
                    className="flex items-center gap-2"
                    disabled={!edits.contents}
                  >
                    <History className="w-4 h-4" />
                    Reset
                  </Button>
                  <ConfirmUpdate
                    previous={{ contents: remote_contents }}
                    content={{ contents: edits.contents }}
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
              <ShowHideButton show={showContents} setShow={setShowContents} />
            </div>
          </CardHeader>
          {showContents && (
            <CardContent className="pr-8">
              <MonacoEditor
                value={edits.contents ?? remote_contents}
                language="dockerfile"
                readOnly={!canEdit}
                onValueChange={(contents) => setEdits({ contents })}
              />
            </CardContent>
          )}
        </Card>
      )}

      {/* Last build output */}
      {last_build && last_build.logs.length > 0 && (
        <code className="font-bold">Last Build Logs</code>
      )}
      {last_build &&
        last_build.logs.length > 0 &&
        last_build.logs?.map((log, i) => (
          <Card key={i}>
            <CardHeader className="flex-col">
              <CardTitle>{log.stage}</CardTitle>
              <CardDescription className="flex gap-2">
                <span>
                  Stage {i + 1} of {last_build.logs.length}
                </span>
                <span>|</span>
                <span className="flex items-center gap-2">
                  <Clock className="w-4 h-4" />
                  {fmt_duration(log.start_ts, log.end_ts)}
                </span>
              </CardDescription>
            </CardHeader>
            <CardContent className="flex flex-col gap-2">
              {log.command && (
                <div>
                  <CardDescription>command</CardDescription>
                  <pre className="max-h-[500px] overflow-y-auto">
                    {log.command}
                  </pre>
                </div>
              )}
              {log.stdout && (
                <div>
                  <CardDescription>stdout</CardDescription>
                  <pre
                    dangerouslySetInnerHTML={{
                      __html: updateLogToHtml(log.stdout),
                    }}
                    className="max-h-[500px] overflow-y-auto"
                  />
                </div>
              )}
              {log.stderr && (
                <div>
                  <CardDescription>stderr</CardDescription>
                  <pre
                    dangerouslySetInnerHTML={{
                      __html: updateLogToHtml(log.stderr),
                    }}
                    className="max-h-[500px] overflow-y-auto"
                  />
                </div>
              )}
            </CardContent>
          </Card>
        ))}
    </Section>
  );
};
