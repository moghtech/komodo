import {
  usePermissions,
  useRead,
  useWebhookIdOrName,
  useWebhookIntegrations,
  useWrite,
} from "@/lib/hooks";
import Config from "@/ui/config";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import { useState } from "react";
import Stage from "./stage";
import { Button, Group, Select, Stack, TextInput } from "@mantine/core";
import { ICONS } from "@/lib/icons";
import { ConfigItem, ConfigSwitch } from "@/ui/config/item";
import TimezoneSelector from "@/components/timezone-selector";
import WebhookBuilder from "@/components/webhook/builder";
import { LabelledSwitch } from "mogh_ui";
import CopyWebhookUrl from "@/components/webhook/copy-url";
import { useFullProcedure } from "..";

const PROCEDURE_GIT_PROVIDER = "Procedure";

export function newStage(next_index: number) {
  return {
    name: `Stage ${next_index}`,
    enabled: true,
    executions: [defaultEnabledExecution()],
  };
}

export function defaultEnabledExecution(): Types.EnabledExecution {
  return {
    enabled: true,
    execution: {
      type: "None",
      params: {},
    },
  };
}

export default function ProcedureConfig({ id }: { id: string }) {
  const [branch, setBranch] = useState("main");
  const { canWrite } = usePermissions({ type: "Procedure", id });
  const procedure = useFullProcedure(id);
  const config = procedure?.config;
  const name = procedure?.name;
  const global_disabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const [update, setUpdate] = useLocalStorage<Partial<Types.ProcedureConfig>>({
    key: `procedure-${id}-update-v1`,
    defaultValue: {},
  });
  const { mutateAsync } = useWrite("UpdateProcedure");
  const { integrations } = useWebhookIntegrations();
  const [idOrName] = useWebhookIdOrName();

  if (!config) return null;

  const disabled = global_disabled || !canWrite;
  const webhookIntegration = integrations[PROCEDURE_GIT_PROVIDER] ?? "Github";
  const stages = update.stages || procedure.config?.stages || [];

  const addStage = () =>
    setUpdate((config) => ({
      ...config,
      stages: [...stages, newStage(stages.length + 1)],
    }));

  return (
    <Config
      disabled={disabled}
      original={config}
      update={update}
      setUpdate={setUpdate}
      onSave={() => mutateAsync({ id, config: update })}
      groups={{
        "": [
          {
            label: "Stages",
            description:
              "The executions in a stage are all run in parallel. The stages themselves are run sequentially.",
            fields: {
              stages: (stages, set) => (
                <Stack gap="xl">
                  {stages?.map((stage, index) => (
                    <Stage
                      key={index}
                      stage={stage}
                      setStage={(stage) =>
                        set({
                          stages: stages.map((s, i) =>
                            index === i ? stage : s,
                          ),
                        })
                      }
                      removeStage={() =>
                        set({
                          stages: stages.filter((_, i) => index !== i),
                        })
                      }
                      moveUp={
                        index === 0
                          ? undefined
                          : () =>
                              set({
                                stages: stages.map((stage, i) => {
                                  // Make sure its not the first row
                                  if (i === index && index !== 0) {
                                    return stages[index - 1];
                                  } else if (i === index - 1) {
                                    // Reverse the entry, moving this row "Up"
                                    return stages[index];
                                  } else {
                                    return stage;
                                  }
                                }),
                              })
                      }
                      moveDown={
                        index === stages.length - 1
                          ? undefined
                          : () =>
                              set({
                                stages: stages.map((stage, i) => {
                                  // The index also cannot be the last index, which cannot be moved down
                                  if (
                                    i === index &&
                                    index !== stages.length - 1
                                  ) {
                                    return stages[index + 1];
                                  } else if (i === index + 1) {
                                    // Move the row "Down"
                                    return stages[index];
                                  } else {
                                    return stage;
                                  }
                                }),
                              })
                      }
                      insertAbove={() =>
                        set({
                          stages: [
                            ...stages.slice(0, index),
                            newStage(index + 1),
                            ...stages.slice(index),
                          ],
                        })
                      }
                      insertBelow={() =>
                        set({
                          stages: [
                            ...stages.slice(0, index + 1),
                            newStage(index + 2),
                            ...stages.slice(index + 1),
                          ],
                        })
                      }
                      disabled={disabled}
                    />
                  ))}
                  <Button
                    leftSection={<ICONS.Add size="1rem" />}
                    onClick={addStage}
                    disabled={disabled}
                    w="fit-content"
                  >
                    Add Stage
                  </Button>
                </Stack>
              ),
            },
          },
          {
            label: "Alert",
            labelHidden: true,
            fields: {
              failure_alert: {
                description: "Send an alert any time the Procedure fails",
              },
            },
          },
          {
            label: "Schedule",
            description:
              "Configure the Procedure to run at defined times using English or CRON.",
            fields: {
              schedule_enabled: (schedule_enabled, set) => (
                <ConfigSwitch
                  label="Enabled"
                  value={
                    (update.schedule ?? config.schedule)
                      ? schedule_enabled
                      : false
                  }
                  disabled={disabled || !(update.schedule ?? config.schedule)}
                  onCheckedChange={(schedule_enabled) =>
                    set({ schedule_enabled })
                  }
                />
              ),
              schedule_format: (schedule_format, set) => (
                <ConfigItem
                  label="Format"
                  description="Choose whether to provide English or CRON schedule expression"
                >
                  <Select
                    value={schedule_format}
                    onChange={(schedule_format) =>
                      schedule_format &&
                      set({
                        schedule_format:
                          schedule_format as Types.ScheduleFormat,
                      })
                    }
                    data={Object.values(Types.ScheduleFormat)}
                    w={{ base: "85%", lg: 400 }}
                  />
                </ConfigItem>
              ),
              schedule: {
                label: "Expression",
                description: (
                  <Stack gap="0" pt="0.2rem">
                    {(update.schedule_format ?? config.schedule_format) ===
                    "Cron" ? (
                      <code>
                        second - minute - hour - day - month - day-of-week
                      </code>
                    ) : (
                      <>
                        <code>- Run every day at 4:00 pm</code>
                        <code>
                          - Run at 21:00 on the 1st and 15th of the month
                        </code>
                        <code>- Every Sunday at midnight</code>
                      </>
                    )}
                  </Stack>
                ),
                placeholder:
                  (update.schedule_format ?? config.schedule_format) === "Cron"
                    ? "0 0 0 ? * SUN"
                    : "Enter English expression",
              },
              schedule_timezone: (timezone, set) => {
                return (
                  <ConfigItem
                    label="Timezone"
                    description="Select specific IANA timezone for schedule expression."
                  >
                    <TimezoneSelector
                      timezone={timezone ?? ""}
                      onChange={(schedule_timezone) =>
                        set({ schedule_timezone })
                      }
                      disabled={disabled}
                    />
                  </ConfigItem>
                );
              },
              schedule_alert: {
                description: "Send an alert when the scheduled run occurs",
              },
            },
          },
          {
            label: "Webhooks",
            description: `Copy the webhook given here, and configure your ${webhookIntegration}-style repo provider to send webhooks to Komodo`,
            fields: {
              ["Builder" as any]: () => (
                <WebhookBuilder
                  gitProvider={PROCEDURE_GIT_PROVIDER}
                  extra={
                    <Group align="end">
                      <TextInput
                        label="Listen on branch?"
                        placeholder="Branch"
                        value={branch}
                        onChange={(e) => setBranch(e.target.value)}
                        w={{ base: "100%", sm: 200 }}
                        disabled={branch === "__ANY__"}
                      />
                      <LabelledSwitch
                        label="No branch check"
                        checked={branch === "__ANY__"}
                        onCheckedChange={(checked) => {
                          if (checked) {
                            setBranch("__ANY__");
                          } else {
                            setBranch("main");
                          }
                        }}
                      />
                    </Group>
                  }
                />
              ),
              ["run" as any]: () => (
                <CopyWebhookUrl
                  label="Webhook URL - Run"
                  integration={webhookIntegration}
                  path={`/procedure/${idOrName === "Id" ? id : encodeURIComponent(name ?? "...")}/${branch}`}
                />
              ),
              webhook_enabled: true,
              webhook_secret: {
                description:
                  "Provide a custom webhook secret for this resource, or use the global default.",
                placeholder: "Input custom secret",
              },
            },
          },
        ],
      }}
    />
  );
}
