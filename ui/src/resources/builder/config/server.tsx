import { usePermissions, useRead, useWrite } from "@/lib/hooks";
import ResourceSelector from "@/resources/selector";
import { Config } from "mogh_ui";
import { ConfigItem } from "mogh_ui";
import { ActionIcon, Button, Group } from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import { ICONS } from "@/lib/icons";

export default function ServerBuilderConfig({ id }: { id: string }) {
  const { canWrite } = usePermissions({ type: "Builder", id });
  const config = useRead("GetBuilder", { builder: id }).data?.config;
  const [update, setUpdate] = useLocalStorage<
    Partial<Types.ServerBuilderConfig>
  >({
    key: `server-builder-${id}-update-v1`,
    defaultValue: {},
  });
  const { mutateAsync } = useWrite("UpdateBuilder");
  if (!config) return null;

  const disabled = !canWrite;

  return (
    <Config
      disabled={disabled}
      original={config.params as Types.ServerBuilderConfig}
      update={update}
      setUpdate={setUpdate}
      onSave={async () => {
        await mutateAsync({ id, config: { type: "Server", params: update } });
      }}
      groups={{
        "": [
          {
            label: "Server",
            labelHidden: true,
            fields: {
              server_ids: (serverIds, set) => {
                return (
                  <ConfigItem
                    label="Select Servers"
                    description="If multiple servers are configured, will distribute builds among them."
                    gap="sm"
                  >
                    {serverIds?.map((serverId, index) => {
                      return (
                        <Group
                          key={index}
                          gap="xs"
                          w={{ base: "100%", lg: 400 }}
                          justify="space-between"
                          wrap="nowrap"
                        >
                          <ResourceSelector
                            type="Server"
                            excludeIds={serverIds}
                            selected={serverId}
                            onSelect={(server_id) =>
                              set({
                                server_ids: [
                                  ...serverIds.map((id, i) =>
                                    i === index ? server_id : id,
                                  ),
                                ],
                              })
                            }
                            disabled={disabled}
                            targetProps={{ w: "90%", maw: "" }}
                            clearable={false}
                          />
                          {!disabled && (
                            <ActionIcon variant="filled" color="red">
                              <ICONS.Remove
                                size="1rem"
                                onClick={() =>
                                  set({
                                    server_ids: [
                                      ...serverIds?.filter(
                                        (_, i) => i !== index,
                                      ),
                                    ],
                                  })
                                }
                              />
                            </ActionIcon>
                          )}
                        </Group>
                      );
                    })}
                    {!disabled && (
                      <Button
                        onClick={() =>
                          set({
                            server_ids: [...(serverIds ?? []), ""],
                          })
                        }
                        leftSection={<ICONS.Add size="1rem" />}
                        w={{ base: "100%", lg: 400 }}
                      >
                        Add Server
                      </Button>
                    )}
                  </ConfigItem>
                );
              },
            },
          },
        ],
      }}
    />
  );
}
