import { usePermissions, useRead, useWrite } from "@/lib/hooks";
import { ReactNode } from "react";
import { useFullSwarm } from ".";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import Config from "@/ui/config";
import { ConfigItem, ConfigList } from "@/ui/config/item";
import { ActionIcon, Button, Group } from "@mantine/core";
import ResourceSelector from "@/resources/selector";
import { ICONS } from "@/theme/icons";

export default function SwarmConfig({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const { canWrite } = usePermissions({ type: "Swarm", id });
  const swarm = useFullSwarm(id);
  const config = swarm?.config;
  const globalDisabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const [update, setUpdate] = useLocalStorage<Partial<Types.SwarmConfig>>({
    key: `swarm-${id}-update-v1`,
    defaultValue: {},
  });
  const { mutateAsync } = useWrite("UpdateSwarm");

  if (!config) return null;

  const disabled = globalDisabled || !canWrite;

  return (
    <Config
      titleOther={titleOther}
      disabled={disabled}
      original={config}
      update={update}
      setUpdate={setUpdate}
      onSave={() => mutateAsync({ id, config: update })}
      groups={{
        "": [
          {
            label: "Managers",
            labelHidden: true,
            fields: {
              server_ids: (serverIds, set) => {
                return (
                  <ConfigItem
                    label="Manager Nodes"
                    boldLabel
                    description="Select the Servers which have joined the Swarm as Manager Nodes."
                    gap="sm"
                  >
                    {serverIds?.map((serverId, index) => {
                      return (
                        <Group
                          key={index}
                          gap="xs"
                          w={{ base: "85%", lg: 400 }}
                          justify="space-between"
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
                        w={{ base: "85%", lg: 400 }}
                      >
                        Add Server
                      </Button>
                    )}
                  </ConfigItem>
                );
              },
            },
          },
          {
            label: "Links",
            description: "Add quick links in the resource header",
            contentHidden: ((update.links ?? config.links)?.length ?? 0) === 0,
            fields: {
              links: (values, set) => (
                <ConfigList
                  field="links"
                  values={values ?? []}
                  set={set}
                  disabled={disabled}
                  placeholder="Input link"
                />
              ),
            },
          },
        ],
      }}
    />
  );
}
