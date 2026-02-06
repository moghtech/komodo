import { Section } from "@components/layouts";
import { Card, CardContent, CardHeader } from "@ui/card";
import { cn, getUpdateQuery, updateLogToHtml } from "@lib/utils";
import { useRead } from "@lib/hooks";
import { text_color_class_by_intention } from "@lib/color";
import { useTheme } from "@ui/theme";

export const ActionInfo = ({ id }: { id: string }) => {
  const { currentTheme } = useTheme();
  const update = useRead("ListUpdates", {
    query: {
      ...getUpdateQuery({ type: "Action", id }, undefined),
      operation: "RunAction",
    },
  }).data?.updates[0];

  const full_update = useRead(
    "GetUpdate",
    { id: update?.id! },
    { enabled: !!update?.id }
  ).data;

  const log = full_update?.logs.find((log) => log.stage === "Execute Action");

  if (!log?.stdout && !log?.stderr) {
    return (
      <Section>
        <Card className="flex flex-col gap-4">
          <CardHeader
            className={cn(
              "flex flex-row justify-between items-center",
              text_color_class_by_intention("Neutral")
            )}
          >
            Never run
          </CardHeader>
        </Card>
      </Section>
    );
  }

  return (
    <Section>
      {/* Last run */}
      {log?.stdout && (
        <Card className="flex flex-col gap-4">
          <CardHeader className="flex flex-row items-center gap-1 pb-0">
            Last run -
            <div className={text_color_class_by_intention("Good")}>Stdout</div>
          </CardHeader>
          <CardContent className="pr-8">
            <pre
              dangerouslySetInnerHTML={{
                __html: updateLogToHtml(log.stdout, currentTheme),
              }}
              className="max-h-[500px] overflow-y-auto"
            />
          </CardContent>
        </Card>
      )}
      {log?.stderr && (
        <Card className="flex flex-col gap-4">
          <CardHeader className="flex flex-row items-center gap-1 pb-0">
            Last run -
            <div className={text_color_class_by_intention("Critical")}>
              Stderr
            </div>
          </CardHeader>
          <CardContent className="pr-8">
            <pre
              dangerouslySetInnerHTML={{
                __html: updateLogToHtml(log.stderr, currentTheme),
              }}
              className="max-h-[500px] overflow-y-auto"
            />
          </CardContent>
        </Card>
      )}
    </Section>
  );
};
