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
import { ConfigItem, WebhookBuilder } from "@components/config/util";
import { Input } from "@ui/input";
import { useState } from "react";
import { CopyWebhook } from "../common";
import { ActionInfo } from "./info";
import { Switch } from "@ui/switch";

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
      disableSidebar
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
            // actions: (
            //   <ShowHideButton
            //     show={show.file}
            //     setShow={(file) => setShow({ ...show, file })}
            //   />
            // ),
            // contentHidden: !show.file,
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
