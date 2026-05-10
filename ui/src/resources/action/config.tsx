import { Anchor, Group, Select, Stack, Text, TextInput } from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { useState } from "react";
import { Types } from "komodo_client";
import {
  usePermissions,
  useRead,
  useWebhookIdOrName,
  useWebhookIntegrations,
  useWrite,
} from "@/lib/hooks";
import { fmtSnakeCaseToUpperSpaceCase } from "mogh_ui";
import { MonacoEditor } from "mogh_ui";
import { Config, ConfigItem, ConfigSwitch } from "mogh_ui";
import ActionLastRun from "./last-run";
import TimezoneSelector from "@/components/timezone-selector";
import SecretsSearch from "@/components/config/secrets-search";
import WebhookBuilder from "@/components/webhook/builder";
import { LabelledSwitch } from "mogh_ui";
import CopyWebhookUrl from "@/components/webhook/copy-url";
import { useFullAction } from ".";

const ACTION_GIT_PROVIDER = "Action";

export default function ActionConfig({ id }: { id: string }) {
  const [branch, setBranch] = useState("main");
  const { canWrite } = usePermissions({ type: "Action", id });
  const action = useFullAction(id);
  const config = action?.config;
  const name = action?.name;
  const global_disabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const [update, setUpdate] = useLocalStorage<Partial<Types.ActionConfig>>({
    key: `action-${id}-update-v1`,
    defaultValue: {},
  });
  const { mutateAsync } = useWrite("UpdateAction");
  const { integrations } = useWebhookIntegrations();
  const [idOrName] = useWebhookIdOrName();

  if (!config) return null;

  const disabled = global_disabled || !canWrite;
  const webhookIntegration = integrations.Action ?? "Github";

  return (
    <Config
      disabled={disabled}
      original={config}
      update={update}
      setUpdate={setUpdate}
      onSave={async () => {
        await mutateAsync({ id, config: update });
      }}
      groups={{
        "": [
          {
            label: "Action File",
            description: "Manage the action file contents here.",
            fields: {
              file_contents(value, set) {
                return (
                  <Stack>
                    <Group justify="space-between">
                      <SecretsSearch />
                      <Group visibleFrom="lg">
                        <Text c="dimmed">Docs</Text>
                        {["read", "execute", "write"].map((api) => (
                          <Anchor
                            key={api}
                            href={`https://docs.rs/komodo_client/latest/komodo_client/api/${api}/index.html`}
                            target="_blank"
                          >
                            {api}
                          </Anchor>
                        ))}
                      </Group>
                    </Group>
                    <MonacoEditor
                      value={value}
                      onValueChange={(file_contents) => set({ file_contents })}
                      language="typescript"
                      readOnly={disabled}
                    />
                    <ActionLastRun id={id} />
                  </Stack>
                );
              },
            },
          },
          {
            label: "Arguments",
            description: "Manage the action file default arguments.",
            fields: {
              arguments(args, set) {
                const format =
                  update.arguments_format ??
                  config.arguments_format ??
                  Types.FileFormat.KeyValue;

                return (
                  <Stack>
                    <Group>
                      <SecretsSearch />
                      <Select
                        placeholder="Select format"
                        value={format}
                        onChange={(format) =>
                          format &&
                          set({ arguments_format: format as Types.FileFormat })
                        }
                        data={Object.values(Types.FileFormat).map((format) => ({
                          value: format,
                          label: fmtSnakeCaseToUpperSpaceCase(format),
                        }))}
                      />
                    </Group>
                    <MonacoEditor
                      value={args || defaultArguments(format)}
                      onValueChange={(args) => set({ arguments: args })}
                      language={
                        update.arguments_format ??
                        config.arguments_format ??
                        Types.FileFormat.KeyValue
                      }
                      readOnly={disabled}
                    />
                  </Stack>
                );
              },
            },
          },
          {
            label: "Alert",
            labelHidden: true,
            fields: {
              failure_alert: {
                description: "Send an alert any time the Action fails",
              },
            },
          },
          {
            label: "Schedule",
            description:
              "Configure the Action to run at defined times using English or CRON.",
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
            label: "Startup",
            labelHidden: true,
            fields: {
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
            fields: {
              reload_deno_deps: {
                label: "Reload Dependencies",
                description:
                  "Whether deno will be instructed to reload all dependencies. This can usually be kept disabled outside of development.",
              },
            },
          },
          {
            label: "Webhooks",
            description: `Copy the webhook given here, and configure your ${webhookIntegration}-style repo provider to send webhooks to Komodo`,
            fields: {
              ["Builder" as any]: () => (
                <WebhookBuilder
                  gitProvider={ACTION_GIT_PROVIDER}
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
                  path={`/action/${idOrName === "Id" ? id : encodeURIComponent(name ?? "...")}/${branch}`}
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

const defaultArguments = (format: Types.FileFormat) => {
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
