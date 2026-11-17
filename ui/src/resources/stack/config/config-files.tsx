import { ICONS } from "@/lib/icons";
import { ConfigItem, languageFromPath, MonacoEditor } from "mogh_ui";
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
import { ChevronDown, ChevronRight } from "lucide-react";
import { useState } from "react";
import { useStack } from "..";
import SecretsSearch from "@/components/config/secrets-search";

export interface StackConfigFilesProps {
  id: string;
  value: Types.StackFileDependency[] | undefined;
  set: (value: Partial<Types.StackConfig>) => void;
  disabled: boolean;
  uiDefined?: boolean;
}

export default function StackConfigFiles({
  id,
  value,
  set,
  disabled,
  uiDefined,
}: StackConfigFilesProps) {
  const values = value ?? [];
  const allServices = useStack(id)?.info.services.map((s) => s.service) ?? [];
  const [openOptions, setOpenOptions] = useState(() =>
    values.flatMap((v, i) => (v.user || v.group || v.mode ? [i] : [])),
  );
  const update = (i: number, file: Partial<Types.StackFileDependency>) =>
    set({
      config_files: values.map((v, index) =>
        i === index ? { ...v, ...file } : v,
      ),
    });
  return (
    <ConfigItem>
      {uiDefined && <SecretsSearch />}
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
                  requires: uiDefined
                    ? Types.StackFileRequires.None
                    : Types.StackFileRequires.Restart,
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
          {values.map(({ path, services, requires, contents, ...file }, i) => {
            const optionsOpen = openOptions.includes(i);
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
                      onClick={() => {
                        set({
                          config_files: [
                            ...values.filter((_, idx) => idx !== i),
                          ],
                        });
                        setOpenOptions(
                          openOptions
                            .filter((idx) => idx !== i)
                            .map((idx) => (idx > i ? idx - 1 : idx)),
                        );
                      }}
                      disabled={disabled}
                    >
                      <ICONS.Remove size="1rem" />
                    </ActionIcon>
                  }
                />

                {/** Permissions toggle */}
                {uiDefined && (
                  <Button
                    variant="default"
                    fw="normal"
                    rightSection={
                      optionsOpen ? (
                        <ChevronDown size="1rem" />
                      ) : (
                        <ChevronRight size="1rem" />
                      )
                    }
                    onClick={() =>
                      setOpenOptions(
                        optionsOpen
                          ? openOptions.filter((idx) => idx !== i)
                          : [...openOptions, i],
                      )
                    }
                  >
                    Permissions
                    {(file.mode || file.user || file.group) &&
                      `: ${[
                        file.mode,
                        (file.user || file.group) &&
                          `${file.user || "-"}:${file.group || "-"}`,
                      ]
                        .filter(Boolean)
                        .join(" · ")}`}
                  </Button>
                )}

                {/** Services / Requires */}
                {!uiDefined && (
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
                )}

                {/** Owner / Mode */}
                {uiDefined && optionsOpen && (
                  <Group w="100%">
                    <TextInput
                      label="User"
                      placeholder="Periphery user id"
                      value={file.user ?? ""}
                      onChange={(e) => update(i, { user: e.target.value })}
                      disabled={disabled}
                      w={150}
                    />
                    <TextInput
                      label="Group"
                      placeholder="Periphery group id"
                      value={file.group ?? ""}
                      onChange={(e) => update(i, { group: e.target.value })}
                      disabled={disabled}
                      w={150}
                    />
                    <TextInput
                      label="Mode"
                      placeholder="0644"
                      value={file.mode ?? ""}
                      onChange={(e) => update(i, { mode: e.target.value })}
                      disabled={disabled}
                      w={100}
                    />
                  </Group>
                )}

                {/** Contents */}
                {uiDefined && (
                  <Stack w="100%">
                    <MonacoEditor
                      value={contents ?? ""}
                      language={languageFromPath(path)}
                      filename={path}
                      onValueChange={(contents) =>
                        set({
                          config_files: values.map((v, index) =>
                            i === index ? { ...v, contents } : v,
                          ),
                        })
                      }
                      readOnly={disabled}
                    />
                  </Stack>
                )}
              </Group>
            );
          })}
        </Stack>
      )}
    </ConfigItem>
  );
}
