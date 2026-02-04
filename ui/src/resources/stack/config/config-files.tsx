import { ICONS } from "@/theme/icons";
import { ConfigItem } from "@/ui/config/item";
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
import { StackComponents } from "..";

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
  const allServices =
    StackComponents.useListItem(id)?.info.services.map((s) => s.service) ?? [];
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
                    values[i] = { ...values[i], path: e.target.value };
                    set({ config_files: [...values] });
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
                    placeholder={
                      services?.length ? "Add services" : "All services"
                    }
                    value={services}
                    data={allServices}
                    onChange={(services) => {
                      values[i] = { ...values[i], services };
                      set({ config_files: [...values] });
                    }}
                    disabled={disabled}
                    searchable
                    clearable
                  />
                  <Select
                    value={requires}
                    onChange={(requires) => {
                      if (!requires) return;
                      values[i] = {
                        ...values[i],
                        requires: requires as Types.StackFileRequires,
                      };
                      set({ config_files: [...values] });
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
