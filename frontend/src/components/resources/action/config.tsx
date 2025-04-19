import {
  useLocalStorage,
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

const ACTION_GIT_PROVIDER = "Action";

export const ActionConfig = ({ id }: { id: string }) => {
  const [branch, setBranch] = useState("main");
  const perms = useRead("GetPermissionLevel", {
    target: { type: "Action", id },
  }).data;
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

  const disabled = global_disabled || perms !== Types.PermissionLevel.Write;
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
              schedule_timezone: {
                label: "Timezone",
                description:
                  "Optional. Enter specific IANA timezone for schedule expression. If not provided, uses the Core timezone.",
                placeholder: "Enter IANA timezone",
              },
              schedule_alert: {
                description: "Send an alert when the scheduled run occurs",
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
                    path={`/action/${id_or_name === "Id" ? id : name}/${branch}`}
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
