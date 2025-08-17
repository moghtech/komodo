import {
  useLocalStorage,
  usePermissions,
  useRead,
  useWebhookIdOrName,
  useWebhookIntegrations,
  useWrite,
} from "@lib/hooks";
import { Types } from "komodo_client";
import { Config } from "@components/config";
import { MonacoEditor } from "@components/monaco";
import { SecretsSearch } from "@components/config/env_vars";
import { Button } from "@ui/button";
import {
  ConfigItem,
  ConfigSwitch,
  WebhookBuilder,
} from "@components/config/util";
import { Input } from "@ui/input";
import { useState } from "react";
import { CopyWebhook } from "../common";
import { ActionInfo } from "./info";
import { Switch } from "@ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { TimezoneSelector } from "@components/util";
import { snake_case_to_upper_space_case } from "@lib/formatting";

const ACTION_GIT_PROVIDER = "Action";

export const ActionConfig = ({ id }: { id: string }) => {
  const [branch, setBranch] = useState("main");
  const { canWrite } = usePermissions({ type: "Action", id });
  const action = useRead("GetAction", { action: id }).data;
  const config = action?.config;
  const name = action?.name;
  const global_disabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const [update, set] = useLocalStorage<Partial<Types.ActionConfig>>(
    `action-${id}-update-v1`,
    {}
  );
  const { mutateAsync } = useWrite("UpdateAction");
  const { integrations } = useWebhookIntegrations();
  const [id_or_name] = useWebhookIdOrName();

  if (!config) return null;

  const disabled = global_disabled || !canWrite;
  const webhook_integration = integrations[ACTION_GIT_PROVIDER] ?? "Github";

  return (
    <Config
      disabled={disabled}
      original={config}
      update={update}
      set={set}
      onSave={async () => {
        await mutateAsync({ id, config: update });
      }}
      components={{
        "": [
          {
            label: "Action File",
            description: "Manage the action file contents here.",
            components: {
              file_contents: (file_contents, set) => {
                return (
                  <div className="flex flex-col gap-4">
                    <div className="flex items-center justify-between">
                      <SecretsSearch />
                      <div className="hidden lg:flex items-center">
                        <div className="text-muted-foreground text-sm mr-2">
                          Docs:
                        </div>
                        {["read", "execute", "write"].map((api) => (
                          <a
                            key={api}
                            href={`https://docs.rs/komodo_client/latest/komodo_client/api/${api}/index.html`}
                            target="_blank"
                          >
                            <Button
                              className="capitalize px-1"
                              size="sm"
                              variant="link"
                            >
                              {api}
                            </Button>
                          </a>
                        ))}
                      </div>
                    </div>
                    <MonacoEditor
                      value={file_contents}
                      onValueChange={(file_contents) => set({ file_contents })}
                      language="typescript"
                      readOnly={disabled}
                    />
                    <ActionInfo id={id} />
                  </div>
                );
              },
            },
          },
          {
            label: "Arguments",
            description: "Manage the action file default arguments.",
            components: {
              arguments: (args, set) => {
                const format =
                  update.arguments_format ??
                  config.arguments_format ??
                  Types.FileFormat.KeyValue;
                return (
                  <div className="flex flex-col gap-4">
                    <div className="flex items-center gap-4">
                      <SecretsSearch />
                      <Select
                        value={format}
                        onValueChange={(arguments_format: Types.FileFormat) =>
                          set({ arguments_format })
                        }
                      >
                        <SelectTrigger className="w-fit">
                          <div className="flex gap-2 items-center mr-2">
                            <div className="text-muted-foreground">Format:</div>
                            <SelectValue />
                          </div>
                        </SelectTrigger>
                        <SelectContent>
                          {Object.values(Types.FileFormat)
                            // Don't allow selection of Toml, as this option will break resource sync
                            .filter((f) => f !== Types.FileFormat.Toml)
                            .map((format) => (
                              <SelectItem value={format}>
                                {snake_case_to_upper_space_case(format)}
                              </SelectItem>
                            ))}
                        </SelectContent>
                      </Select>
                    </div>
                    <MonacoEditor
                      value={args || default_arguments(format)}
                      onValueChange={(args) => set({ arguments: args })}
                      language={
                        update.arguments_format ??
                        config.arguments_format ??
                        Types.FileFormat.KeyValue
                      }
                      readOnly={disabled}
                    />
                  </div>
                );
              },
            },
          },

          {
            label: "Alert",
            labelHidden: true,
            components: {
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
            components: {
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
                    onValueChange={(schedule_format) =>
                      set({
                        schedule_format:
                          schedule_format as Types.ScheduleFormat,
                      })
                    }
                    disabled={disabled}
                  >
                    <SelectTrigger className="w-[200px]" disabled={disabled}>
                      <SelectValue placeholder="Select Format" />
                    </SelectTrigger>
                    <SelectContent>
                      {Object.values(Types.ScheduleFormat).map((mode) => (
                        <SelectItem
                          key={mode}
                          value={mode!}
                          className="cursor-pointer"
                        >
                          {mode}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
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
            label: "Startup",
            labelHidden: true,
            components: {
              run_at_startup: {
                label: "Run on Startup",
                description:
                  "Run this action on completion of startup of Komodo Core",
              },
            },
          },
          {
            label: "Reload",
            labelHidden: true,
            components: {
              reload_deno_deps: {
                label: "Reload Dependencies",
                description:
                  "Whether deno will be instructed to reload all dependencies. This can usually be kept disabled outside of development.",
              },
            },
          },
          {
            label: "Webhook",
            description: `Copy the webhook given here, and configure your ${webhook_integration}-style repo provider to send webhooks to Komodo`,
            components: {
              ["Builder" as any]: () => (
                <WebhookBuilder git_provider={ACTION_GIT_PROVIDER}>
                  <div className="text-nowrap text-muted-foreground text-sm">
                    Listen on branch:
                  </div>
                  <div className="flex items-center gap-3">
                    <Input
                      placeholder="Branch"
                      value={branch}
                      onChange={(e) => setBranch(e.target.value)}
                      className="w-[200px]"
                      disabled={branch === "__ANY__"}
                    />
                    <div className="flex items-center gap-2">
                      <div className="text-muted-foreground text-sm">
                        No branch check:
                      </div>
                      <Switch
                        checked={branch === "__ANY__"}
                        onCheckedChange={(checked) => {
                          if (checked) {
                            setBranch("__ANY__");
                          } else {
                            setBranch("main");
                          }
                        }}
                      />
                    </div>
                  </div>
                </WebhookBuilder>
              ),
              ["run" as any]: () => (
                <ConfigItem label="Webhook Url - Run">
                  <CopyWebhook
                    integration={webhook_integration}
                    path={`/action/${id_or_name === "Id" ? id : encodeURIComponent(name ?? "...")}/${branch}`}
                  />
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
};

const default_arguments = (format: Types.FileFormat) => {
  switch (format) {
    case Types.FileFormat.KeyValue:
      return "# ARG_NAME = value\n";
    case Types.FileFormat.Toml:
      return '# ARG_NAME = "value"\n';
    case Types.FileFormat.Yaml:
      return "# ARG_NAME: value\n";
    case Types.FileFormat.Json:
      return "{}\n";
  }
};
