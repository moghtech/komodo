import { usePermissions, useRead, useWrite } from "@/lib/hooks";
import ResourceLink from "@/resources/link";
import ResourceSelector from "@/resources/selector";
import { Config } from "mogh_ui";
import { ConfigItem } from "mogh_ui";
import { Group } from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";

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
              server_id: (serverId, set) => {
                return (
                  <ConfigItem
                    label={
                      serverId ? (
                        <Group fz="h3" fw="bold">
                          Server:
                          <ResourceLink
                            type="Server"
                            id={serverId}
                            fz="h3"
                            iconSize="1.2rem"
                          />
                        </Group>
                      ) : (
                        "Select Server"
                      )
                    }
                    description="Select the Server to build on."
                  >
                    <ResourceSelector
                      type="Server"
                      selected={serverId}
                      onSelect={(server_id) => set({ server_id })}
                      disabled={disabled}
                      clearable
                    />
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
