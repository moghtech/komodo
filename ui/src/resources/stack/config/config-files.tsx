import { ICONS } from "@/lib/icons";
import { ConfigItem } from "mogh_ui";
import {
  ActionIcon,
  Button,
  Group,
  MultiSelect,
  Select,
  Stack,
  TextInput,
} from "@mantine/core";
import { Types } from "komodo_client";
import { useStack } from "..";

export interface StackConfigFilesProps {
  id: string;
  value: Types.StackFileDependency[] | undefined;
  set: (value: Partial<Types.StackConfig>) => void;
  disabled: boolean;
}

export default function StackConfigFiles({
  id,
  value,
  set,
  disabled,
}: StackConfigFilesProps) {
  const values = value ?? [];
  const allServices = useStack(id)?.info.services.map((s) => s.service) ?? [];
  return (
    <ConfigItem>
      {!disabled && (
        <Button
          leftSection={<ICONS.Add size="1rem" />}
          onClick={() =>
            set({
              config_files: [
                ...values,
                {
                  path: "",
                  services: [],
                  requires: Types.StackFileRequires.Restart,
                },
              ],
            })
          }
          w={{ base: "85%", lg: 400 }}
        >
          Add File
        </Button>
      )}
      {values.length > 0 && (
        <Stack>
          {values.map(({ path, services, requires }, i) => {
            return (
              <Group key={i}>
                {/** Path */}
                <TextInput
                  placeholder="configs/config.yaml"
                  value={path}
                  onChange={(e) => {
                    set({
                      config_files: values.map((v, index) =>
                        i === index ? { ...v, path: e.target.value } : v,
                      ),
                    });
                  }}
                  w={{ base: "100%", md: 400 }}
                  disabled={disabled}
                  rightSection={
                    <ActionIcon
                      color="red"
                      onClick={() =>
                        set({
                          config_files: [
                            ...values.filter((_, idx) => idx !== i),
                          ],
                        })
                      }
                      disabled={disabled}
                    >
                      <ICONS.Remove size="1rem" />
                    </ActionIcon>
                  }
                />

                {/** Services / Requires */}
                <Group>
                  <MultiSelect
                    leftSection={<ICONS.Service size="1rem" />}
                    placeholder={
                      services?.length ? "Add services" : "All services"
                    }
                    value={services}
                    data={allServices}
                    onChange={(services) => {
                      set({
                        config_files: values.map((v, index) =>
                          i === index ? { ...v, services } : v,
                        ),
                      });
                    }}
                    disabled={disabled}
                    searchable
                    clearable
                  />
                  <Select
                    value={requires}
                    onChange={(requires) => {
                      if (!requires) return;
                      set({
                        config_files: values.map((v, index) =>
                          i === index
                            ? {
                                ...v,
                                requires: requires as Types.StackFileRequires,
                              }
                            : v,
                        ),
                      });
                    }}
                    disabled={disabled}
                    data={Object.values(Types.StackFileRequires)}
                  />
                </Group>
              </Group>
            );
          })}
        </Stack>
      )}
    </ConfigItem>
  );
}
