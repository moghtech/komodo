import { DataTable } from "@/ui/data-table";
import {
  ActionIcon,
  Button,
  Group,
  Menu,
  Stack,
  Switch,
  TextInput,
} from "@mantine/core";
import { Types } from "komodo_client";
import { defaultEnabledExecution } from ".";
import { ICONS } from "@/lib/icons";
import ProcedureExecutionSelector from "./execution-selector";
import { PROCEDURE_EXECUTIONS, ProcedureMinExecutionType } from "./executions";
import { EnableSwitch } from "mogh_ui";
import { ArrowDown, ArrowUp, Ellipsis, Plus, Trash } from "lucide-react";

export interface ProcedureStageProps {
  stage: Types.ProcedureStage;
  setStage: (stage: Types.ProcedureStage) => void;
  removeStage: () => void;
  insertAbove: () => void;
  insertBelow: () => void;
  moveUp: (() => void) | undefined;
  moveDown: (() => void) | undefined;
  disabled: boolean;
}

export default function ProcedureStage({
  stage,
  setStage,
  removeStage,
  moveUp,
  moveDown,
  insertAbove,
  insertBelow,
  disabled,
}: ProcedureStageProps) {
  return (
    <Stack>
      <Group>
        <TextInput
          value={stage.name}
          onChange={(e) => setStage({ ...stage, name: e.target.value })}
          w={250}
        />
        <Group>
          <EnableSwitch
            checked={stage.enabled}
            onCheckedChange={(enabled) => setStage({ ...stage, enabled })}
          />
          <Menu>
            <Menu.Target>
              <ActionIcon aria-label="Open menu">
                <Ellipsis size="1rem" />
              </ActionIcon>
            </Menu.Target>
            <Menu.Dropdown>
              {moveUp && (
                <Menu.Item
                  rightSection={<ArrowUp size="1rem" />}
                  onClick={moveUp}
                >
                  Move Up
                </Menu.Item>
              )}
              {moveDown && (
                <Menu.Item
                  rightSection={<ArrowDown size="1rem" />}
                  onClick={moveDown}
                >
                  Move Down
                </Menu.Item>
              )}

              {(moveUp ?? moveDown) && <Menu.Divider />}

              <Menu.Item
                rightSection={
                  <Group gap="0.1rem">
                    <ArrowUp size="1rem" />
                    <Plus size="1rem" />
                  </Group>
                }
                onClick={insertAbove}
              >
                Insert Above
              </Menu.Item>
              <Menu.Item
                rightSection={
                  <Group gap="0.1rem">
                    <ArrowDown size="1rem" />
                    <Plus size="1rem" />
                  </Group>
                }
                onClick={insertBelow}
              >
                Insert Below
              </Menu.Item>

              <Menu.Divider />

              <Menu.Item
                rightSection={<Trash size="1rem" />}
                onClick={removeStage}
              >
                Remove
              </Menu.Item>
            </Menu.Dropdown>
          </Menu>
        </Group>
      </Group>

      <DataTable
        tableKey="procedure-stage-executions"
        data={stage.executions!}
        noResults={
          <Button
            onClick={() =>
              setStage({
                ...stage,
                executions: [defaultEnabledExecution()],
              })
            }
            disabled={disabled}
          >
            Add Execution
          </Button>
        }
        columns={[
          {
            header: "Execution",
            size: 250,
            cell: ({ row: { original, index } }) => (
              <ProcedureExecutionSelector
                disabled={disabled}
                type={original.execution.type}
                onSelect={(type) =>
                  setStage({
                    ...stage,
                    executions: stage.executions!.map((item, i) =>
                      i === index
                        ? ({
                            ...item,
                            execution: {
                              type,
                              params:
                                PROCEDURE_EXECUTIONS[
                                  type as ProcedureMinExecutionType
                                ].params,
                            },
                          } as Types.EnabledExecution)
                        : item,
                    ),
                  })
                }
              />
            ),
          },
          {
            header: "Target",
            size: 250,
            cell: ({
              row: {
                original: {
                  execution: { type, params },
                },
                index,
              },
            }) => {
              const Component =
                PROCEDURE_EXECUTIONS[type as ProcedureMinExecutionType]
                  .Component;
              return (
                <Component
                  disabled={disabled}
                  params={params as any}
                  setParams={(params: any) =>
                    setStage({
                      ...stage,
                      executions: stage.executions!.map((item, i) =>
                        i === index
                          ? {
                              ...item,
                              execution: { type, params },
                            }
                          : item,
                      ) as Types.EnabledExecution[],
                    })
                  }
                />
              );
            },
          },
          {
            header: "Add / Remove",
            size: 150,
            cell: ({ row: { index } }) => (
              <Group>
                <ActionIcon
                  variant="secondary"
                  onClick={() =>
                    setStage({
                      ...stage,
                      executions: [
                        ...stage.executions!.slice(0, index + 1),
                        defaultEnabledExecution(),
                        ...stage.executions!.slice(index + 1),
                      ],
                    })
                  }
                  disabled={disabled}
                >
                  <ICONS.Add size="1rem" />
                </ActionIcon>
                <ActionIcon
                  onClick={() =>
                    setStage({
                      ...stage,
                      executions: stage.executions!.filter(
                        (_, i) => i !== index,
                      ),
                    })
                  }
                  disabled={disabled}
                >
                  <ICONS.Remove size="1rem" />
                </ActionIcon>
              </Group>
            ),
          },
          {
            header: "Enabled",
            size: 100,
            cell: ({
              row: {
                original: { enabled },
                index,
              },
            }) => {
              return (
                <Switch
                  checked={enabled}
                  onClick={() =>
                    setStage({
                      ...stage,
                      executions: stage.executions!.map((item, i) =>
                        i === index ? { ...item, enabled: !enabled } : item,
                      ),
                    })
                  }
                  disabled={disabled}
                />
              );
            },
          },
        ]}
      />
    </Stack>
  );
}
