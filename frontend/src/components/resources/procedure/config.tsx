import {
  useLocalStorage,
  usePermissions,
  useRead,
  useWebhookIdOrName,
  useWebhookIntegrations,
  useWrite,
} from "@lib/hooks";
import { Types } from "komodo_client";
import { Config } from "@components/config";
import { Button } from "@ui/button";
import {
  ConfigItem,
  ConfigSwitch,
  WebhookBuilder,
} from "@components/config/util";
import { Input } from "@ui/input";
import { useEffect, useState } from "react";
import { CopyWebhook, ResourceSelector } from "../common";
import { Switch } from "@ui/switch";
import {
  ArrowDown,
  ArrowUp,
  ChevronsUpDown,
  Minus,
  MinusCircle,
  Plus,
  PlusCircle,
  SearchX,
  Settings2,
  CheckCircle,
} from "lucide-react";
import { useToast } from "@ui/use-toast";
import { TextUpdateMenuMonaco, TimezoneSelector } from "@components/util";
import { Card } from "@ui/card";
import { cn, filterBySplit, text_to_env } from "@lib/utils";
import { Popover, PopoverContent, PopoverTrigger } from "@ui/popover";
import { fmt_upper_camelcase } from "@lib/formatting";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@ui/command";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@ui/dropdown-menu";
import { DotsHorizontalIcon } from "@radix-ui/react-icons";
import { DataTable } from "@ui/data-table";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@ui/dialog";
import { MonacoEditor } from "@components/monaco";
import { quote as shellQuote, parse as shellParse } from "shell-quote";

type ExecutionType = Types.Execution["type"];

type ExecutionConfigComponent<
  T extends ExecutionType,
  P = Extract<Types.Execution, { type: T }>["params"],
> = React.FC<{
  params: P;
  setParams: React.Dispatch<React.SetStateAction<P>>;
  disabled: boolean;
}>;

type MinExecutionType = Exclude<
  ExecutionType,
  | "StartContainer"
  | "RestartContainer"
  | "PauseContainer"
  | "UnpauseContainer"
  | "StopContainer"
  | "DestroyContainer"
  | "DeleteNetwork"
  | "DeleteImage"
  | "DeleteVolume"
  | "TestAlerter"
  | "RemoveSwarmNodes"
  | "RemoveSwarmStacks"
  | "RemoveSwarmServices"
  | "RemoveSwarmConfigs"
  | "RemoveSwarmSecrets"
>;

type ExecutionConfigParams<T extends MinExecutionType> = Extract<
  Types.Execution,
  { type: T }
>["params"];

type ExecutionConfigs = {
  [ExType in MinExecutionType]: {
    Component: ExecutionConfigComponent<ExType>;
    params: ExecutionConfigParams<ExType>;
  };
};

const PROCEDURE_GIT_PROVIDER = "Procedure";

const new_stage = (next_index: number) => ({
  name: `Stage ${next_index}`,
  enabled: true,
  executions: [default_enabled_execution()],
});

const default_enabled_execution: () => Types.EnabledExecution = () => ({
  enabled: true,
  execution: {
    type: "None",
    params: {},
  },
});

export const ProcedureConfig = ({ id }: { id: string }) => {
  const [branch, setBranch] = useState("main");
  const { canWrite } = usePermissions({ type: "Procedure", id });
  const procedure = useRead("GetProcedure", { procedure: id }).data;
  const config = procedure?.config;
  const name = procedure?.name;
  const global_disabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const [update, set] = useLocalStorage<Partial<Types.ProcedureConfig>>(
    `procedure-${id}-update-v1`,
    {}
  );
  const { mutateAsync } = useWrite("UpdateProcedure");
  const { integrations } = useWebhookIntegrations();
  const [id_or_name] = useWebhookIdOrName();

  if (!config) return null;

  const disabled = global_disabled || !canWrite;
  const webhook_integration = integrations[PROCEDURE_GIT_PROVIDER] ?? "Github";
  const stages = update.stages || procedure.config?.stages || [];

  const add_stage = () =>
    set((config) => ({
      ...config,
      stages: [...stages, new_stage(stages.length + 1)],
    }));

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
            label: "Stages",
            description:
              "The executions in a stage are all run in parallel. The stages themselves are run sequentially.",
            components: {
              stages: (stages, set) => (
                <div className="flex flex-col gap-4">
                  {stages?.map((stage, index) => (
                    <Stage
                      key={index}
                      stage={stage}
                      setStage={(stage) =>
                        set({
                          stages: stages.map((s, i) =>
                            index === i ? stage : s
                          ),
                        })
                      }
                      removeStage={() =>
                        set({
                          stages: stages.filter((_, i) => index !== i),
                        })
                      }
                      moveUp={
                        index === 0
                          ? undefined
                          : () =>
                              set({
                                stages: stages.map((stage, i) => {
                                  // Make sure its not the first row
                                  if (i === index && index !== 0) {
                                    return stages[index - 1];
                                  } else if (i === index - 1) {
                                    // Reverse the entry, moving this row "Up"
                                    return stages[index];
                                  } else {
                                    return stage;
                                  }
                                }),
                              })
                      }
                      moveDown={
                        index === stages.length - 1
                          ? undefined
                          : () =>
                              set({
                                stages: stages.map((stage, i) => {
                                  // The index also cannot be the last index, which cannot be moved down
                                  if (
                                    i === index &&
                                    index !== stages.length - 1
                                  ) {
                                    return stages[index + 1];
                                  } else if (i === index + 1) {
                                    // Move the row "Down"
                                    return stages[index];
                                  } else {
                                    return stage;
                                  }
                                }),
                              })
                      }
                      insertAbove={() =>
                        set({
                          stages: [
                            ...stages.slice(0, index),
                            new_stage(index + 1),
                            ...stages.slice(index),
                          ],
                        })
                      }
                      insertBelow={() =>
                        set({
                          stages: [
                            ...stages.slice(0, index + 1),
                            new_stage(index + 2),
                            ...stages.slice(index + 1),
                          ],
                        })
                      }
                      disabled={disabled}
                    />
                  ))}
                  <Button
                    variant="secondary"
                    onClick={add_stage}
                    className="w-fit"
                    disabled={disabled}
                  >
                    Add Stage
                  </Button>
                </div>
              ),
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
            label: "Webhook",
            description: `Copy the webhook given here, and configure your ${webhook_integration}-style repo provider to send webhooks to Komodo`,
            components: {
              ["Builder" as any]: () => (
                <WebhookBuilder git_provider={PROCEDURE_GIT_PROVIDER}>
                  <div className="text-nowrap text-muted-foreground text-sm">
                    Listen on branch:
                  </div>
                  <div className="flex items-center gap-3 flex-wrap">
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
                    path={`/procedure/${id_or_name === "Id" ? id : encodeURIComponent(name ?? "...")}/${branch}`}
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

const Stage = ({
  stage,
  setStage,
  removeStage,
  moveUp,
  moveDown,
  insertAbove,
  insertBelow,
  disabled,
}: {
  stage: Types.ProcedureStage;
  setStage: (stage: Types.ProcedureStage) => void;
  removeStage: () => void;
  insertAbove: () => void;
  insertBelow: () => void;
  moveUp: (() => void) | undefined;
  moveDown: (() => void) | undefined;
  disabled: boolean;
}) => {
  return (
    <Card className="p-4 flex flex-col gap-4">
      <div className="flex justify-between items-center gap-2 flex-wrap">
        <Input
          value={stage.name}
          onChange={(e) => setStage({ ...stage, name: e.target.value })}
          className="w-[250px] text-md"
        />
        <div className="flex gap-4 items-center">
          <div>Enabled:</div>
          <Switch
            checked={stage.enabled}
            onCheckedChange={(enabled) => setStage({ ...stage, enabled })}
          />
          <DropdownMenu>
            <DropdownMenuTrigger asChild disabled={disabled}>
              <Button
                variant="ghost"
                className="h-8 w-8 p-0"
                disabled={disabled}
              >
                <span className="sr-only">Open menu</span>
                <DotsHorizontalIcon className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              {moveUp && (
                <DropdownMenuItem
                  className="flex gap-4 justify-between cursor-pointer"
                  onClick={moveUp}
                >
                  Move Up <ArrowUp className="w-4 h-4" />
                </DropdownMenuItem>
              )}
              {moveDown && (
                <DropdownMenuItem
                  className="flex gap-4 justify-between cursor-pointer"
                  onClick={moveDown}
                >
                  Move Down <ArrowDown className="w-4 h-4" />
                </DropdownMenuItem>
              )}

              {(moveUp ?? moveDown) && <DropdownMenuSeparator />}

              <DropdownMenuItem
                className="flex gap-4 justify-between cursor-pointer"
                onClick={insertAbove}
              >
                Insert Above{" "}
                <div className="flex">
                  <ArrowUp className="w-4 h-4" />
                  <Plus className="w-4 h-4" />
                </div>
              </DropdownMenuItem>
              <DropdownMenuItem
                className="flex gap-4 justify-between cursor-pointer"
                onClick={insertBelow}
              >
                Insert Below{" "}
                <div className="flex">
                  <ArrowDown className="w-4 h-4" />
                  <Plus className="w-4 h-4" />
                </div>
              </DropdownMenuItem>

              <DropdownMenuSeparator />

              <DropdownMenuItem
                className="flex gap-4 justify-between cursor-pointer"
                onClick={removeStage}
              >
                Remove <Minus className="w-4 h-4" />
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
      <DataTable
        tableKey="procedure-stage-executions"
        data={stage.executions!}
        noResults={
          <Button
            onClick={() =>
              setStage({
                ...stage,
                executions: [default_enabled_execution()],
              })
            }
            variant="secondary"
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
              <ExecutionTypeSelector
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
                                TARGET_COMPONENTS[type as MinExecutionType]
                                  .params,
                            },
                          } as Types.EnabledExecution)
                        : item
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
                TARGET_COMPONENTS[type as MinExecutionType].Component;
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
                          : item
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
              <div className="flex items-center gap-2">
                <Button
                  variant="secondary"
                  onClick={() =>
                    setStage({
                      ...stage,
                      executions: [
                        ...stage.executions!.slice(0, index + 1),
                        default_enabled_execution(),
                        ...stage.executions!.slice(index + 1),
                      ],
                    })
                  }
                  disabled={disabled}
                >
                  <PlusCircle className="w-4 h-4" />
                </Button>
                <Button
                  variant="secondary"
                  onClick={() =>
                    setStage({
                      ...stage,
                      executions: stage.executions!.filter(
                        (_, i) => i !== index
                      ),
                    })
                  }
                  disabled={disabled}
                >
                  <MinusCircle className="w-4 h-4" />
                </Button>
              </div>
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
                        i === index ? { ...item, enabled: !enabled } : item
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
    </Card>
  );
};

const ExecutionTypeSelector = ({
  type,
  onSelect,
  disabled,
}: {
  type: Types.Execution["type"];
  onSelect: (type: Types.Execution["type"]) => void;
  disabled: boolean;
}) => {
  const execution_types = Object.keys(TARGET_COMPONENTS).filter(
    (c) => !["None"].includes(c)
  );

  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const filtered = filterBySplit(execution_types, search, (item) => item);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button variant="secondary" className="flex gap-2" disabled={disabled}>
          {fmt_upper_camelcase(type)}
          <ChevronsUpDown className="w-3 h-3" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[200px] max-h-[200px] p-0" sideOffset={12}>
        <Command shouldFilter={false}>
          <CommandInput
            placeholder="Search Executions"
            className="h-9"
            value={search}
            onValueChange={setSearch}
          />
          <CommandList>
            <CommandEmpty className="flex justify-evenly items-center pt-2">
              Empty.
              <SearchX className="w-3 h-3" />
            </CommandEmpty>
            <CommandGroup className="overflow-auto">
              {filtered.map((type) => (
                <CommandItem
                  key={type}
                  onSelect={() => onSelect(type as Types.Execution["type"])}
                  className="flex items-center justify-between"
                >
                  <div className="p-1">{fmt_upper_camelcase(type)}</div>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
};

const TARGET_COMPONENTS: ExecutionConfigs = {
  None: {
    params: {},
    Component: () => <></>,
  },
  // Procedure
  RunProcedure: {
    params: { procedure: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Procedure"
        selected={params.procedure}
        onSelect={(procedure) => setParams({ procedure })}
        disabled={disabled}
      />
    ),
  },
  BatchRunProcedure: {
    params: { pattern: "" },
    Component: ({ params, setParams, disabled }) => (
      <TextUpdateMenuMonaco
        title="Match procedures"
        value={
          params.pattern ||
          "# Match procedures by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        language="string_list"
        fullWidth
      />
    ),
  },
  // Action
  RunAction: {
    params: { action: "", args: {} },
    Component: ({ params, setParams, disabled }) => (
      <div className="flex gap-2 items-center">
        <ResourceSelector
          type="Action"
          selected={params.action}
          onSelect={(action) => setParams({ action, args: params.args })}
          disabled={disabled}
        />
        <TextUpdateMenuMonaco
          title="Action Arguments (JSON)"
          value={JSON.stringify(params.args ?? {}, undefined, 2)}
          onUpdate={(args) =>
            setParams({ action: params.action, args: JSON.parse(args) })
          }
          disabled={disabled}
          language="json"
          triggerChild={
            <Button variant="secondary" size="icon">
              <Settings2 className="w-4" />
            </Button>
          }
        />
      </div>
    ),
  },
  BatchRunAction: {
    params: { pattern: "" },
    Component: ({ params, setParams, disabled }) => (
      <TextUpdateMenuMonaco
        title="Match actions"
        value={
          params.pattern ||
          "# Match actions by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        language="string_list"
        fullWidth
      />
    ),
  },
  // Build
  RunBuild: {
    params: { build: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Build"
        selected={params.build}
        onSelect={(build) => setParams({ build })}
        disabled={disabled}
      />
    ),
  },
  BatchRunBuild: {
    params: { pattern: "" },
    Component: ({ params, setParams, disabled }) => (
      <TextUpdateMenuMonaco
        title="Match builds"
        value={
          params.pattern ||
          "# Match builds by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        language="string_list"
        fullWidth
      />
    ),
  },
  CancelBuild: {
    params: { build: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Build"
        selected={params.build}
        onSelect={(build) => setParams({ build })}
        disabled={disabled}
      />
    ),
  },
  // Deployment
  Deploy: {
    params: { deployment: "" },
    Component: ({ params, setParams, disabled }) => {
      return (
        <ResourceSelector
          type="Deployment"
          selected={params.deployment}
          onSelect={(deployment) => setParams({ deployment })}
          disabled={disabled}
        />
      );
    },
  },
  BatchDeploy: {
    params: { pattern: "" },
    Component: ({ params, setParams, disabled }) => (
      <TextUpdateMenuMonaco
        title="Match deployments"
        value={
          params.pattern ||
          "# Match deployments by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        language="string_list"
        fullWidth
      />
    ),
  },
  PullDeployment: {
    params: { deployment: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Deployment"
        selected={params.deployment}
        onSelect={(deployment) => setParams({ deployment })}
        disabled={disabled}
      />
    ),
  },
  StartDeployment: {
    params: { deployment: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Deployment"
        selected={params.deployment}
        onSelect={(deployment) => setParams({ deployment })}
        disabled={disabled}
      />
    ),
  },
  RestartDeployment: {
    params: { deployment: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Deployment"
        selected={params.deployment}
        onSelect={(deployment) => setParams({ deployment })}
        disabled={disabled}
      />
    ),
  },
  PauseDeployment: {
    params: { deployment: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Deployment"
        selected={params.deployment}
        onSelect={(deployment) => setParams({ deployment })}
        disabled={disabled}
      />
    ),
  },
  UnpauseDeployment: {
    params: { deployment: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Deployment"
        selected={params.deployment}
        onSelect={(deployment) => setParams({ deployment })}
        disabled={disabled}
      />
    ),
  },
  StopDeployment: {
    params: { deployment: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Deployment"
        selected={params.deployment}
        onSelect={(id) => setParams({ deployment: id })}
        disabled={disabled}
      />
    ),
  },
  DestroyDeployment: {
    params: { deployment: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Deployment"
        selected={params.deployment}
        onSelect={(deployment) => setParams({ deployment })}
        disabled={disabled}
      />
    ),
  },
  BatchDestroyDeployment: {
    params: { pattern: "" },
    Component: ({ params, setParams, disabled }) => (
      <TextUpdateMenuMonaco
        title="Match deployments"
        value={
          params.pattern ||
          "# Match deployments by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        language="string_list"
        fullWidth
      />
    ),
  },
  // Stack
  DeployStack: {
    params: { stack: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Stack"
        selected={params.stack}
        onSelect={(id) => setParams({ stack: id })}
        disabled={disabled}
      />
    ),
  },
  BatchDeployStack: {
    params: { pattern: "" },
    Component: ({ params, setParams, disabled }) => (
      <TextUpdateMenuMonaco
        title="Match stacks"
        value={
          params.pattern ||
          "# Match stacks by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        language="string_list"
        fullWidth
      />
    ),
  },
  DeployStackIfChanged: {
    params: { stack: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Stack"
        selected={params.stack}
        onSelect={(id) => setParams({ stack: id })}
        disabled={disabled}
      />
    ),
  },
  BatchDeployStackIfChanged: {
    params: { pattern: "" },
    Component: ({ params, setParams, disabled }) => (
      <TextUpdateMenuMonaco
        title="Match stacks"
        value={
          params.pattern ||
          "# Match stacks by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        language="string_list"
        fullWidth
      />
    ),
  },
  PullStack: {
    params: { stack: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Stack"
        selected={params.stack}
        onSelect={(id) => setParams({ stack: id })}
        disabled={disabled}
      />
    ),
  },
  BatchPullStack: {
    params: { pattern: "" },
    Component: ({ params, setParams, disabled }) => (
      <TextUpdateMenuMonaco
        title="Match stacks"
        value={
          params.pattern ||
          "# Match stacks by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        language="string_list"
        fullWidth
      />
    ),
  },
  StartStack: {
    params: { stack: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Stack"
        selected={params.stack}
        onSelect={(id) => setParams({ stack: id })}
        disabled={disabled}
      />
    ),
  },
  RestartStack: {
    params: { stack: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Stack"
        selected={params.stack}
        onSelect={(id) => setParams({ stack: id })}
        disabled={disabled}
      />
    ),
  },
  PauseStack: {
    params: { stack: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Stack"
        selected={params.stack}
        onSelect={(id) => setParams({ stack: id })}
        disabled={disabled}
      />
    ),
  },
  UnpauseStack: {
    params: { stack: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Stack"
        selected={params.stack}
        onSelect={(id) => setParams({ stack: id })}
        disabled={disabled}
      />
    ),
  },
  StopStack: {
    params: { stack: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Stack"
        selected={params.stack}
        onSelect={(id) => setParams({ stack: id })}
        disabled={disabled}
      />
    ),
  },
  DestroyStack: {
    params: { stack: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Stack"
        selected={params.stack}
        onSelect={(id) => setParams({ stack: id })}
        disabled={disabled}
      />
    ),
  },
  BatchDestroyStack: {
    params: { pattern: "" },
    Component: ({ params, setParams, disabled }) => (
      <TextUpdateMenuMonaco
        title="Match stacks"
        value={
          params.pattern ||
          "# Match stacks by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        language="string_list"
        fullWidth
      />
    ),
  },
  RunStackService: {
    params: {
      stack: "",
      service: "",
      command: undefined,
      no_tty: undefined,
      no_deps: undefined,
      detach: undefined,
      service_ports: undefined,
      env: undefined,
      workdir: undefined,
      user: undefined,
      entrypoint: undefined,
      pull: undefined,
    },
    Component: ({ params, setParams, disabled }) => {
      const [open, setOpen] = useState(false);
      // local mirrors to allow cancel without committing
      const [stack, setStack] = useState(params.stack ?? "");
      const [service, setService] = useState(params.service ?? "");
      const [commandText, setCommand] = useState(
        params.command && params.command.length
          ? shellQuote(params.command)
          : ""
      );
      const [no_tty, setNoTty] = useState(!!params.no_tty);
      const [no_deps, setNoDeps] = useState(!!params.no_deps);
      const [detach, setDetach] = useState(!!params.detach);
      const [service_ports, setServicePorts] = useState(!!params.service_ports);
      const [workdir, setWorkdir] = useState(params.workdir ?? "");
      const [user, setUser] = useState(params.user ?? "");
      const [entrypoint, setEntrypoint] = useState(params.entrypoint ?? "");
      const [pull, setPull] = useState(!!params.pull);
      const env_text = (
        params.env
          ? Object.entries(params.env)
              .map(([k, v]) => `${k}=${v}`)
              .join("\n")
          : "  # VARIABLE = value\n"
      ) as string;
      const [envText, setEnvText] = useState(env_text);

      useEffect(() => {
        setStack(params.stack ?? "");
        setService(params.service ?? "");
        setCommand(
          params.command && params.command.length
            ? shellQuote(params.command)
            : ""
        );
        setNoTty(!!params.no_tty);
        setNoDeps(!!params.no_deps);
        setDetach(!!params.detach);
        setServicePorts(!!params.service_ports);
        setWorkdir(params.workdir ?? "");
        setUser(params.user ?? "");
        setEntrypoint(params.entrypoint ?? "");
        setPull(!!params.pull);
        setEnvText(
          params.env
            ? Object.entries(params.env)
                .map(([k, v]) => `${k}=${v}`)
                .join("\n")
            : "  # VARIABLE = value\n"
        );
      }, [params]);

      const onConfirm = () => {
        const envArray = text_to_env(envText);
        const env = envArray.length
          ? envArray.reduce<Record<string, string>>(
              (acc, { variable, value }) => {
                if (variable) acc[variable] = value;
                return acc;
              },
              {}
            )
          : undefined;
        const parsed = commandText.trim()
          ? shellParse(commandText.trim()).map((tok) =>
              typeof tok === "string" ? tok : ((tok as any).op ?? String(tok))
            )
          : [];
        setParams({
          stack,
          service,
          command: parsed.length ? (parsed as string[]) : undefined,
          no_tty: no_tty ? true : undefined,
          no_deps: no_deps ? true : undefined,
          service_ports: service_ports ? true : undefined,
          workdir: workdir || undefined,
          user: user || undefined,
          entrypoint: entrypoint || undefined,
          pull: pull ? true : undefined,
          detach: detach ? true : undefined,
          env,
        } as any);
        setOpen(false);
      };

      return (
        <Dialog open={open} onOpenChange={setOpen}>
          <DialogTrigger asChild>
            <Button variant="secondary" disabled={disabled}>
              Configure
            </Button>
          </DialogTrigger>
          <DialogContent className="w-[800px] max-w-[95vw] max-h-[80vh] overflow-auto">
            <DialogHeader>
              <DialogTitle>Run Stack Service</DialogTitle>
            </DialogHeader>
            <div className="flex flex-col gap-3 py-2">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                <div className="flex flex-col gap-1">
                  <div className="text-muted-foreground text-xs">Stack</div>
                  <ResourceSelector
                    type="Stack"
                    selected={stack}
                    onSelect={(id) => setStack(id)}
                    disabled={disabled}
                    align="start"
                  />
                </div>
                <div className="flex flex-col gap-1">
                  <div className="text-muted-foreground text-xs">Service</div>
                  <Input
                    placeholder="service name"
                    value={service}
                    onChange={(e) => setService(e.target.value)}
                    disabled={disabled}
                  />
                </div>
              </div>

              <div className="flex flex-col gap-1">
                <div className="text-muted-foreground text-xs">Command</div>
                <Input
                  value={commandText}
                  onChange={(e) => setCommand(e.target.value)}
                  disabled={disabled}
                />
              </div>

              <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                <label className="flex items-center gap-2">
                  <Switch checked={no_tty} onCheckedChange={setNoTty} />
                  <span className="text-sm">No TTY</span>
                </label>
                <label className="flex items-center gap-2">
                  <Switch checked={no_deps} onCheckedChange={setNoDeps} />
                  <span className="text-sm">No Dependencies</span>
                </label>
                <label className="flex items-center gap-2">
                  <Switch checked={detach} onCheckedChange={setDetach} />
                  <span className="text-sm">Detach</span>
                </label>
                <label className="flex items-center gap-2">
                  <Switch
                    checked={service_ports}
                    onCheckedChange={setServicePorts}
                  />
                  <span className="text-sm">Service Ports</span>
                </label>
                <label className="flex items-center gap-2">
                  <Switch checked={pull} onCheckedChange={setPull} />
                  <span className="text-sm">Pull Image</span>
                </label>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                <div className="flex flex-col gap-1">
                  <div className="text-muted-foreground text-xs">
                    Working Directory
                  </div>
                  <Input
                    placeholder="/work/dir"
                    value={workdir}
                    onChange={(e) => setWorkdir(e.target.value)}
                    disabled={disabled}
                  />
                </div>
                <div className="flex flex-col gap-1">
                  <div className="text-muted-foreground text-xs">User</div>
                  <Input
                    placeholder="uid:gid or user"
                    value={user}
                    onChange={(e) => setUser(e.target.value)}
                    disabled={disabled}
                  />
                </div>
                <div className="flex flex-col gap-1 md:col-span-2">
                  <div className="text-muted-foreground text-xs">
                    Entrypoint
                  </div>
                  <Input
                    value={entrypoint}
                    onChange={(e) => setEntrypoint(e.target.value)}
                    disabled={disabled}
                  />
                </div>
              </div>

              <div className="flex flex-col gap-1">
                <div className="text-muted-foreground text-xs">
                  Extra Environment Variables
                </div>
                <MonacoEditor
                  value={envText}
                  onValueChange={setEnvText}
                  language="key_value"
                  readOnly={disabled}
                />
              </div>
            </div>
            <DialogFooter>
              <Button
                variant="secondary"
                disabled={disabled}
                onClick={onConfirm}
                className="flex items-center gap-2"
              >
                <CheckCircle className="w-4 h-4" /> Confirm
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      );
    },
  },
  // Repo
  CloneRepo: {
    params: { repo: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Repo"
        selected={params.repo}
        onSelect={(repo) => setParams({ repo })}
        disabled={disabled}
      />
    ),
  },
  BatchCloneRepo: {
    params: { pattern: "" },
    Component: ({ params, setParams, disabled }) => (
      <TextUpdateMenuMonaco
        title="Match repos"
        value={
          params.pattern ||
          "# Match repos by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        language="string_list"
        fullWidth
      />
    ),
  },
  PullRepo: {
    params: { repo: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Repo"
        selected={params.repo}
        onSelect={(repo) => setParams({ repo })}
        disabled={disabled}
      />
    ),
  },
  BatchPullRepo: {
    params: { pattern: "" },
    Component: ({ params, setParams, disabled }) => (
      <TextUpdateMenuMonaco
        title="Match repos"
        value={
          params.pattern ||
          "# Match repos by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        language="string_list"
        fullWidth
      />
    ),
  },
  BuildRepo: {
    params: { repo: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Repo"
        selected={params.repo}
        onSelect={(repo) => setParams({ repo })}
        disabled={disabled}
      />
    ),
  },
  BatchBuildRepo: {
    params: { pattern: "" },
    Component: ({ params, setParams, disabled }) => (
      <TextUpdateMenuMonaco
        title="Match repos"
        value={
          params.pattern ||
          "# Match repos by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        language="string_list"
        fullWidth
      />
    ),
  },
  CancelRepoBuild: {
    params: { repo: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Repo"
        selected={params.repo}
        onSelect={(repo) => setParams({ repo })}
        disabled={disabled}
      />
    ),
  },
  // Server
  // StartContainer: {
  //   params: { server: "" },
  //   Component: ({ params, setParams, disabled }) => (
  //     <ResourceSelector
  //       type="Server"
  //       selected={params.server}
  //       onSelect={(server) => setParams({ server })}
  //       disabled={disabled}
  //     />
  //   ),
  // },
  // RestartContainer: {
  //   params: { server: "" },
  //   Component: ({ params, setParams, disabled }) => (
  //     <ResourceSelector
  //       type="Server"
  //       selected={params.server}
  //       onSelect={(server) => setParams({ server })}
  //       disabled={disabled}
  //     />
  //   ),
  // },
  // PauseContainer: {
  //   params: { server: "" },
  //   Component: ({ params, setParams, disabled }) => (
  //     <ResourceSelector
  //       type="Server"
  //       selected={params.server}
  //       onSelect={(server) => setParams({ server })}
  //       disabled={disabled}
  //     />
  //   ),
  // },
  // UnpauseContainer: {
  //   params: { server: "" },
  //   Component: ({ params, setParams, disabled }) => (
  //     <ResourceSelector
  //       type="Server"
  //       selected={params.server}
  //       onSelect={(server) => setParams({ server })}
  //       disabled={disabled}
  //     />
  //   ),
  // },
  // StopContainer: {
  //   params: { server: "" },
  //   Component: ({ params, setParams, disabled }) => (
  //     <ResourceSelector
  //       type="Server"
  //       selected={params.server}
  //       onSelect={(server) => setParams({ server })}
  //       disabled={disabled}
  //     />
  //   ),
  // },
  // DestroyContainer: {
  //   params: { server: "", container: "" },
  //   Component: ({ params, setParams, disabled }) => (
  //     <ResourceSelector
  //       type="Server"
  //       selected={params.server}
  //       onSelect={(server) => setParams({ server })}
  //       disabled={disabled}
  //     />
  //   ),
  // },
  StartAllContainers: {
    params: { server: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Server"
        selected={params.server}
        onSelect={(id) => setParams({ server: id })}
        disabled={disabled}
      />
    ),
  },
  RestartAllContainers: {
    params: { server: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Server"
        selected={params.server}
        onSelect={(id) => setParams({ server: id })}
        disabled={disabled}
      />
    ),
  },
  PauseAllContainers: {
    params: { server: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Server"
        selected={params.server}
        onSelect={(id) => setParams({ server: id })}
        disabled={disabled}
      />
    ),
  },
  UnpauseAllContainers: {
    params: { server: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Server"
        selected={params.server}
        onSelect={(id) => setParams({ server: id })}
        disabled={disabled}
      />
    ),
  },
  StopAllContainers: {
    params: { server: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Server"
        selected={params.server}
        onSelect={(id) => setParams({ server: id })}
        disabled={disabled}
      />
    ),
  },
  PruneContainers: {
    params: { server: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Server"
        selected={params.server}
        onSelect={(server) => setParams({ server })}
        disabled={disabled}
      />
    ),
  },
  PruneNetworks: {
    params: { server: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Server"
        selected={params.server}
        onSelect={(server) => setParams({ server })}
        disabled={disabled}
      />
    ),
  },
  PruneImages: {
    params: { server: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Server"
        selected={params.server}
        onSelect={(server) => setParams({ server })}
        disabled={disabled}
      />
    ),
  },
  PruneVolumes: {
    params: { server: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Server"
        selected={params.server}
        onSelect={(server) => setParams({ server })}
        disabled={disabled}
      />
    ),
  },
  PruneDockerBuilders: {
    params: { server: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Server"
        selected={params.server}
        onSelect={(server) => setParams({ server })}
        disabled={disabled}
      />
    ),
  },
  PruneBuildx: {
    params: { server: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Server"
        selected={params.server}
        onSelect={(server) => setParams({ server })}
        disabled={disabled}
      />
    ),
  },
  PruneSystem: {
    params: { server: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="Server"
        selected={params.server}
        onSelect={(server) => setParams({ server })}
        disabled={disabled}
      />
    ),
  },
  RunSync: {
    params: { sync: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="ResourceSync"
        selected={params.sync}
        onSelect={(id) => setParams({ sync: id })}
        disabled={disabled}
      />
    ),
  },
  CommitSync: {
    params: { sync: "" },
    Component: ({ params, setParams, disabled }) => (
      <ResourceSelector
        type="ResourceSync"
        selected={params.sync}
        onSelect={(id) => setParams({ sync: id })}
        disabled={disabled}
      />
    ),
  },

  ClearRepoCache: {
    params: {},
    Component: () => <></>,
  },
  BackupCoreDatabase: {
    params: {},
    Component: () => <></>,
  },
  GlobalAutoUpdate: {
    params: {},
    Component: () => <></>,
  },
  RotateAllServerKeys: {
    params: {},
    Component: () => <></>,
  },
  RotateCoreKeys: {
    params: {},
    Component: ({ params, setParams, disabled }) => (
      <div
        className={cn("flex items-center gap-2", !disabled && "cursor-pointer")}
        onClick={() => {
          if (!disabled) {
            setParams({ force: !params.force });
          }
        }}
      >
        Force:
        <Switch checked={params.force} disabled={disabled} />
      </div>
    ),
  },

  SendAlert: {
    params: { message: "" },
    Component: ({ params, setParams, disabled }) => (
      <TextUpdateMenuMonaco
        title="Alert message"
        value={params.message}
        placeholder="Configure custom alert message"
        onUpdate={(message) => setParams({ message })}
        disabled={disabled}
        language={undefined}
        fullWidth
      />
    ),
  },

  Sleep: {
    params: { duration_ms: 0 },
    Component: ({ params, setParams, disabled }) => {
      const { toast } = useToast();
      const [internal, setInternal] = useState(
        params.duration_ms?.toString() ?? ""
      );
      useEffect(() => {
        setInternal(params.duration_ms?.toString() ?? "");
      }, [params.duration_ms]);
      return (
        <Input
          placeholder="Duration in milliseconds"
          value={internal}
          onChange={(e) => setInternal(e.target.value)}
          onBlur={() => {
            const duration_ms = Number(internal);
            if (duration_ms) {
              setParams({ duration_ms });
            } else {
              toast({
                title: "Duration must be valid number",
                variant: "destructive",
              });
            }
          }}
          disabled={disabled}
        />
      );
    },
  },
};
