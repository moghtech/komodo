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
import { Button, Select } from "@mantine/core";
import { ICONS } from "@/theme/icons";
import { ConfigItem, ConfigSwitch } from "@/ui/config/item";

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
  const procedure = useRead("GetProcedure", { procedure: id }).data;
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
  const [id_or_name] = useWebhookIdOrName();

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
                <div className="flex flex-col gap-4">
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
                  >
                    Add Stage
                  </Button>
                </div>
              ),
            },
          },
          {
            label: "Alert",
            labelHidden: true,
            fields: {
              failure_alert: {
                boldLabel: true,
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
                  onChange={(schedule_enabled) => set({ schedule_enabled })}
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
                  />
                </ConfigItem>
              ),
              schedule: {
                label: "Expression",
                description:
                  (update.schedule_format ?? config.schedule_format) ===
                  "Cron" ? (
                    <div className="pt-1 flex flex-col gap-1">
                      <code>
                        second - minute - hour - day - month - day-of-week
                      </code>
                    </div>
                  ) : (
                    <div className="pt-1 flex flex-col gap-1">
                      <code>Examples:</code>
                      <code>- Run every day at 4:00 pm</code>
                      <code>
                        - Run at 21:00 on the 1st and 15th of the month
                      </code>
                      <code>- Every Sunday at midnight</code>
                    </div>
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
                    {/* <TimezoneSelector
                      timezone={timezone ?? ""}
                      onChange={(schedule_timezone) =>
                        set({ schedule_timezone })
                      }
                      disabled={disabled}
                    /> */}
                  </ConfigItem>
                );
              },
              schedule_alert: {
                description: "Send an alert when the scheduled run occurs",
              },
            },
          },
          {
            label: "Webhook",
            description: `Copy the webhook given here, and configure your ${webhookIntegration}-style repo provider to send webhooks to Komodo`,
            fields: {
              // ["Builder" as any]: () => (
              //   <WebhookBuilder git_provider={PROCEDURE_GIT_PROVIDER}>
              //     <div className="text-nowrap text-muted-foreground text-sm">
              //       Listen on branch:
              //     </div>
              //     <div className="flex items-center gap-3 flex-wrap">
              //       <Input
              //         placeholder="Branch"
              //         value={branch}
              //         onChange={(e) => setBranch(e.target.value)}
              //         className="w-[200px]"
              //         disabled={branch === "__ANY__"}
              //       />
              //       <div className="flex items-center gap-2">
              //         <div className="text-muted-foreground text-sm">
              //           No branch check:
              //         </div>
              //         <Switch
              //           checked={branch === "__ANY__"}
              //           onChange={(checked) => {
              //             if (checked) {
              //               setBranch("__ANY__");
              //             } else {
              //               setBranch("main");
              //             }
              //           }}
              //         />
              //       </div>
              //     </div>
              //   </WebhookBuilder>
              // ),
              ["run" as any]: () => (
                <ConfigItem label="Webhook Url - Run">
                  {/* <CopyWebhook
                    integration={webhook_integration}
                    path={`/procedure/${id_or_name === "Id" ? id : encodeURIComponent(name ?? "...")}/${branch}`}
                  /> */}
                </ConfigItem>
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
