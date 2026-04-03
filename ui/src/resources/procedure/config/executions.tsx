import ContainerSelector from "@/components/docker/container-selector";
import { MonacoEditor } from "@/components/monaco";
import StackServiceSelector from "@/components/stack-service-selector";
import { useRead } from "@/lib/hooks";
import { textToEnv } from "@/lib/utils";
import ResourceSelector from "@/resources/selector";
import { ICONS } from "@/lib/icons";
import EnableSwitch from "@/ui/enable-switch";
import TextUpdateModal from "@/ui/text-update-modal";
import {
  Button,
  Group,
  Modal,
  MultiSelect,
  SimpleGrid,
  Stack,
  Switch,
  Text,
  TextInput,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { notifications } from "@mantine/notifications";
import { Types } from "komodo_client";
import { CheckCircle } from "lucide-react";
import { useEffect, useState } from "react";
import { quote as shellQuote, parse as shellParse } from "shell-quote";

export type ExecutionType = Types.Execution["type"];

export type ProcedureExecutionComponent<
  T extends ExecutionType,
  P = Extract<Types.Execution, { type: T }>["params"],
> = React.FC<{
  params: P;
  setParams: React.Dispatch<React.SetStateAction<P>>;
  disabled: boolean;
}>;

export type ProcedureMinExecutionType = Exclude<
  ExecutionType,
  | "DeleteNetwork"
  | "DeleteImage"
  | "DeleteVolume"
  | "TestAlerter"
  | "RemoveSwarmNodes"
  | "UpdateSwarmNode"
  | "RemoveSwarmStacks"
  | "RemoveSwarmServices"
  | "CreateSwarmConfig"
  | "RotateSwarmConfig"
  | "RemoveSwarmConfigs"
  | "CreateSwarmSecret"
  | "RotateSwarmSecret"
  | "RemoveSwarmSecrets"
>;

export type ProcedureExecutionParams<T extends ProcedureMinExecutionType> =
  Extract<Types.Execution, { type: T }>["params"];

export type ProcedureExecutions = {
  [ExType in ProcedureMinExecutionType]: {
    Component: ProcedureExecutionComponent<ExType>;
    params: ProcedureExecutionParams<ExType>;
  };
};

export const PROCEDURE_EXECUTIONS: ProcedureExecutions = {
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
      <TextUpdateModal
        title="Match procedures"
        value={
          params.pattern ||
          "# Match procedures by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        useMonaco
        monacoLanguage="string_list"
      />
    ),
  },
  // Action
  RunAction: {
    params: { action: "", args: {} },
    Component: ({ params, setParams, disabled }) => (
      <Group>
        <ResourceSelector
          type="Action"
          selected={params.action}
          onSelect={(action) => setParams({ action, args: params.args })}
          disabled={disabled}
        />
        <TextUpdateModal
          title="Action Arguments (JSON)"
          value={JSON.stringify(params.args ?? {}, undefined, 2)}
          onUpdate={(args) =>
            setParams({ action: params.action, args: JSON.parse(args) })
          }
          disabled={disabled}
          useMonaco
          monacoLanguage="json"
        />
      </Group>
    ),
  },
  BatchRunAction: {
    params: { pattern: "" },
    Component: ({ params, setParams, disabled }) => (
      <TextUpdateModal
        title="Match actions"
        value={
          params.pattern ||
          "# Match actions by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        useMonaco
        monacoLanguage="string_list"
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
      <TextUpdateModal
        title="Match builds"
        value={
          params.pattern ||
          "# Match builds by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        useMonaco
        monacoLanguage="string_list"
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
      <TextUpdateModal
        title="Match deployments"
        value={
          params.pattern ||
          "# Match deployments by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        useMonaco
        monacoLanguage="string_list"
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
      <TextUpdateModal
        title="Match deployments"
        value={
          params.pattern ||
          "# Match deployments by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        useMonaco
        monacoLanguage="string_list"
      />
    ),
  },
  // Stack
  DeployStack: {
    params: { stack: "", services: [] },
    Component: ({ params, setParams, disabled }) => {
      const allServices = useRead("ListStackServices", {
        stack: params.stack,
      }).data?.map((s) => s.service);
      return (
        <Group>
          <ResourceSelector
            type="Stack"
            selected={params.stack}
            onSelect={(id) =>
              setParams(
                id ? { ...params, stack: id } : { stack: id, services: [] },
              )
            }
            disabled={disabled}
          />
          <MultiSelect
            leftSection={<ICONS.Service size="1rem" />}
            placeholder={params.services?.length ? undefined : "All services"}
            value={params.services}
            data={allServices}
            onChange={(services) => setParams({ ...params, services })}
            styles={{ inputField: { width: 130 } }}
            searchable
            clearable
          />
        </Group>
      );
    },
  },
  BatchDeployStack: {
    params: { pattern: "" },
    Component: ({ params, setParams, disabled }) => (
      <TextUpdateModal
        title="Match stacks"
        value={
          params.pattern ||
          "# Match stacks by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        useMonaco
        monacoLanguage="string_list"
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
      <TextUpdateModal
        title="Match stacks"
        value={
          params.pattern ||
          "# Match stacks by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        useMonaco
        monacoLanguage="string_list"
      />
    ),
  },
  PullStack: {
    params: { stack: "" },
    Component: ({ params, setParams, disabled }) => {
      const allServices = useRead("ListStackServices", {
        stack: params.stack,
      }).data?.map((s) => s.service);
      return (
        <Group>
          <ResourceSelector
            type="Stack"
            selected={params.stack}
            onSelect={(id) =>
              setParams(
                id ? { ...params, stack: id } : { stack: id, services: [] },
              )
            }
            disabled={disabled}
          />
          <MultiSelect
            leftSection={<ICONS.Service size="1rem" />}
            placeholder={params.services?.length ? undefined : "All services"}
            value={params.services}
            data={allServices}
            onChange={(services) => setParams({ ...params, services })}
            styles={{ inputField: { width: 130 } }}
            searchable
            clearable
          />
        </Group>
      );
    },
  },
  BatchPullStack: {
    params: { pattern: "" },
    Component: ({ params, setParams, disabled }) => (
      <TextUpdateModal
        title="Match stacks"
        value={
          params.pattern ||
          "# Match stacks by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        useMonaco
        monacoLanguage="string_list"
      />
    ),
  },
  StartStack: {
    params: { stack: "" },
    Component: ({ params, setParams, disabled }) => {
      const allServices = useRead("ListStackServices", {
        stack: params.stack,
      }).data?.map((s) => s.service);
      return (
        <Group>
          <ResourceSelector
            type="Stack"
            selected={params.stack}
            onSelect={(id) =>
              setParams(
                id ? { ...params, stack: id } : { stack: id, services: [] },
              )
            }
            disabled={disabled}
          />
          <MultiSelect
            leftSection={<ICONS.Service size="1rem" />}
            placeholder={params.services?.length ? undefined : "All services"}
            value={params.services}
            data={allServices}
            onChange={(services) => setParams({ ...params, services })}
            styles={{ inputField: { width: 130 } }}
            searchable
            clearable
          />
        </Group>
      );
    },
  },
  RestartStack: {
    params: { stack: "" },
    Component: ({ params, setParams, disabled }) => {
      const allServices = useRead("ListStackServices", {
        stack: params.stack,
      }).data?.map((s) => s.service);
      return (
        <Group>
          <ResourceSelector
            type="Stack"
            selected={params.stack}
            onSelect={(id) =>
              setParams(
                id ? { ...params, stack: id } : { stack: id, services: [] },
              )
            }
            disabled={disabled}
          />
          <MultiSelect
            leftSection={<ICONS.Service size="1rem" />}
            placeholder={params.services?.length ? undefined : "All services"}
            value={params.services}
            data={allServices}
            onChange={(services) => setParams({ ...params, services })}
            styles={{ inputField: { width: 130 } }}
            searchable
            clearable
          />
        </Group>
      );
    },
  },
  PauseStack: {
    params: { stack: "" },
    Component: ({ params, setParams, disabled }) => {
      const allServices = useRead("ListStackServices", {
        stack: params.stack,
      }).data?.map((s) => s.service);
      return (
        <Group>
          <ResourceSelector
            type="Stack"
            selected={params.stack}
            onSelect={(id) =>
              setParams(
                id ? { ...params, stack: id } : { stack: id, services: [] },
              )
            }
            disabled={disabled}
          />
          <MultiSelect
            leftSection={<ICONS.Service size="1rem" />}
            placeholder={params.services?.length ? undefined : "All services"}
            value={params.services}
            data={allServices}
            onChange={(services) => setParams({ ...params, services })}
            styles={{ inputField: { width: 130 } }}
            searchable
            clearable
          />
        </Group>
      );
    },
  },
  UnpauseStack: {
    params: { stack: "" },
    Component: ({ params, setParams, disabled }) => {
      const allServices = useRead("ListStackServices", {
        stack: params.stack,
      }).data?.map((s) => s.service);
      return (
        <Group>
          <ResourceSelector
            type="Stack"
            selected={params.stack}
            onSelect={(id) =>
              setParams(
                id ? { ...params, stack: id } : { stack: id, services: [] },
              )
            }
            disabled={disabled}
          />
          <MultiSelect
            leftSection={<ICONS.Service size="1rem" />}
            placeholder={params.services?.length ? undefined : "All services"}
            value={params.services}
            data={allServices}
            onChange={(services) => setParams({ ...params, services })}
            styles={{ inputField: { width: 130 } }}
            searchable
            clearable
          />
        </Group>
      );
    },
  },
  StopStack: {
    params: { stack: "" },
    Component: ({ params, setParams, disabled }) => {
      const allServices = useRead("ListStackServices", {
        stack: params.stack,
      }).data?.map((s) => s.service);
      return (
        <Group>
          <ResourceSelector
            type="Stack"
            selected={params.stack}
            onSelect={(id) =>
              setParams(
                id ? { ...params, stack: id } : { stack: id, services: [] },
              )
            }
            disabled={disabled}
          />
          <MultiSelect
            leftSection={<ICONS.Service size="1rem" />}
            placeholder={params.services?.length ? undefined : "All services"}
            value={params.services}
            data={allServices}
            onChange={(services) => setParams({ ...params, services })}
            styles={{ inputField: { width: 130 } }}
            searchable
            clearable
          />
        </Group>
      );
    },
  },
  DestroyStack: {
    params: { stack: "" },
    Component: ({ params, setParams, disabled }) => {
      const allServices = useRead("ListStackServices", {
        stack: params.stack,
      }).data?.map((s) => s.service);
      return (
        <Group>
          <ResourceSelector
            type="Stack"
            selected={params.stack}
            onSelect={(id) =>
              setParams(
                id ? { ...params, stack: id } : { stack: id, services: [] },
              )
            }
            disabled={disabled}
          />
          <MultiSelect
            leftSection={<ICONS.Service size="1rem" />}
            placeholder={params.services?.length ? undefined : "All services"}
            value={params.services}
            data={allServices}
            onChange={(services) => setParams({ ...params, services })}
            styles={{ inputField: { width: 130 } }}
            searchable
            clearable
          />
        </Group>
      );
    },
  },
  BatchDestroyStack: {
    params: { pattern: "" },
    Component: ({ params, setParams, disabled }) => (
      <TextUpdateModal
        title="Match stacks"
        value={
          params.pattern ||
          "# Match stacks by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        useMonaco
        monacoLanguage="string_list"
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
      const [opened, { open, close }] = useDisclosure();
      // local mirrors to allow cancel without committing
      const [stack, setStack] = useState(params.stack ?? "");
      const [service, setService] = useState(params.service ?? "");
      const [commandText, setCommand] = useState(
        params.command && params.command.length
          ? shellQuote(params.command)
          : "",
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
            : "",
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
            : "  # VARIABLE = value\n",
        );
      }, [params]);

      const onConfirm = () => {
        const envArray = textToEnv(envText);
        const env = envArray.length
          ? envArray.reduce<Record<string, string>>(
              (acc, { variable, value }) => {
                if (variable) acc[variable] = value;
                return acc;
              },
              {},
            )
          : undefined;
        const parsed = commandText.trim()
          ? shellParse(commandText.trim()).map((tok) =>
              typeof tok === "string" ? tok : ((tok as any).op ?? String(tok)),
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
        close();
      };

      return (
        <>
          <Button disabled={disabled} onClick={open}>
            Configure
          </Button>

          <Modal
            opened={opened}
            onClose={close}
            title="Run Stack Service"
            size="lg"
          >
            <Stack gap="lg">
              <SimpleGrid cols={{ base: 1, sm: 2 }}>
                <Stack gap="0">
                  <Text>Stack</Text>
                  <ResourceSelector
                    type="Stack"
                    selected={stack}
                    onSelect={(id) => setStack(id)}
                    disabled={disabled}
                    width="target"
                    targetProps={{ w: "100%" }}
                  />
                </Stack>

                <Stack gap="0">
                  <Text>Service</Text>
                  <StackServiceSelector
                    stackId={stack}
                    selected={service}
                    onSelect={setService}
                    disabled={disabled}
                    width="target"
                    targetProps={{ w: "100%" }}
                  />
                </Stack>
              </SimpleGrid>

              <Stack gap="0">
                <Text>Command</Text>
                <TextInput
                  placeholder="Enter command (Optional)"
                  value={commandText}
                  onChange={(e) => setCommand(e.target.value)}
                  disabled={disabled}
                />
              </Stack>

              <SimpleGrid cols={{ base: 1, sm: 2 }}>
                <Stack gap="0">
                  <Text>Working Directory</Text>
                  <TextInput
                    placeholder="/work/dir (Optional)"
                    value={workdir}
                    onChange={(e) => setWorkdir(e.target.value)}
                    disabled={disabled}
                  />
                </Stack>
                <Stack gap="0">
                  <Text>User</Text>
                  <TextInput
                    placeholder="uid:gid or user (Optional)"
                    value={user}
                    onChange={(e) => setUser(e.target.value)}
                    disabled={disabled}
                  />
                </Stack>
              </SimpleGrid>

              <Stack gap="0">
                <Text>Entrypoint</Text>
                <TextInput
                  placeholder="Custom entrypoint (Optional)"
                  value={entrypoint}
                  onChange={(e) => setEntrypoint(e.target.value)}
                  disabled={disabled}
                />
              </Stack>

              <Stack gap="0">
                <Text>Extra Env</Text>
                <MonacoEditor
                  value={envText}
                  onValueChange={setEnvText}
                  language="key_value"
                  readOnly={disabled}
                />
              </Stack>

              <Stack gap="0">
                <Text>Options</Text>
                <SimpleGrid
                  cols={{ base: 1, sm: 2 }}
                  className="accent-hover-light"
                  p="md"
                  bdrs="md"
                  style={{ placeItems: "center" }}
                >
                  <EnableSwitch
                    label="No TTY"
                    checked={no_tty}
                    onCheckedChange={setNoTty}
                    disabled={disabled}
                    labelProps={{
                      w: 210,
                      justify: "end",
                    }}
                  />
                  <EnableSwitch
                    label="No Dependencies"
                    checked={no_deps}
                    onCheckedChange={setNoDeps}
                    disabled={disabled}
                    labelProps={{
                      w: 210,
                      justify: "end",
                    }}
                  />
                  <EnableSwitch
                    label="Detach"
                    checked={detach}
                    onCheckedChange={setDetach}
                    disabled={disabled}
                    labelProps={{
                      w: 210,
                      justify: "end",
                    }}
                  />
                  <EnableSwitch
                    label="Service Ports"
                    checked={service_ports}
                    onCheckedChange={setServicePorts}
                    disabled={disabled}
                    labelProps={{
                      w: 210,
                      justify: "end",
                    }}
                  />
                  <EnableSwitch
                    label="Pull Image"
                    checked={pull}
                    onCheckedChange={setPull}
                    disabled={disabled}
                    labelProps={{
                      w: 210,
                      justify: "end",
                    }}
                  />
                </SimpleGrid>
              </Stack>

              {!disabled && (
                <Button onClick={onConfirm} leftSection={<CheckCircle />}>
                  Confirm
                </Button>
              )}
            </Stack>
          </Modal>
        </>
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
      <TextUpdateModal
        title="Match repos"
        value={
          params.pattern ||
          "# Match repos by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        useMonaco
        monacoLanguage="string_list"
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
      <TextUpdateModal
        title="Match repos"
        value={
          params.pattern ||
          "# Match repos by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        useMonaco
        monacoLanguage="string_list"
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
      <TextUpdateModal
        title="Match repos"
        value={
          params.pattern ||
          "# Match repos by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        useMonaco
        monacoLanguage="string_list"
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
  StartContainer: {
    params: { server: "", container: "" },
    Component: ({ params, setParams, disabled }) => (
      <Group>
        <ResourceSelector
          type="Server"
          selected={params.server}
          onSelect={(server) =>
            setParams({ server, container: params.container })
          }
          disabled={disabled}
        />
        <ContainerSelector
          serverId={params.server}
          selected={params.container}
          onSelect={(container) =>
            setParams({ server: params.server, container })
          }
          disabled={disabled}
        />
      </Group>
    ),
  },
  RestartContainer: {
    params: { server: "", container: "" },
    Component: ({ params, setParams, disabled }) => (
      <Group>
        <ResourceSelector
          type="Server"
          selected={params.server}
          onSelect={(server) =>
            setParams({ server, container: params.container })
          }
          disabled={disabled}
        />
        <ContainerSelector
          serverId={params.server}
          selected={params.container}
          onSelect={(container) =>
            setParams({ server: params.server, container })
          }
          disabled={disabled}
        />
      </Group>
    ),
  },
  PauseContainer: {
    params: { server: "", container: "" },
    Component: ({ params, setParams, disabled }) => (
      <Group>
        <ResourceSelector
          type="Server"
          selected={params.server}
          onSelect={(server) =>
            setParams({ server, container: params.container })
          }
          disabled={disabled}
        />
        <ContainerSelector
          serverId={params.server}
          selected={params.container}
          onSelect={(container) =>
            setParams({ server: params.server, container })
          }
          disabled={disabled}
        />
      </Group>
    ),
  },
  UnpauseContainer: {
    params: { server: "", container: "" },
    Component: ({ params, setParams, disabled }) => (
      <Group>
        <ResourceSelector
          type="Server"
          selected={params.server}
          onSelect={(server) =>
            setParams({ server, container: params.container })
          }
          disabled={disabled}
        />
        <ContainerSelector
          serverId={params.server}
          selected={params.container}
          onSelect={(container) =>
            setParams({ server: params.server, container })
          }
          disabled={disabled}
        />
      </Group>
    ),
  },
  StopContainer: {
    params: { server: "", container: "" },
    Component: ({ params, setParams, disabled }) => (
      <Group>
        <ResourceSelector
          type="Server"
          selected={params.server}
          onSelect={(server) =>
            setParams({ server, container: params.container })
          }
          disabled={disabled}
        />
        <ContainerSelector
          serverId={params.server}
          selected={params.container}
          onSelect={(container) =>
            setParams({ server: params.server, container })
          }
          disabled={disabled}
        />
      </Group>
    ),
  },
  DestroyContainer: {
    params: { server: "", container: "" },
    Component: ({ params, setParams, disabled }) => (
      <Group>
        <ResourceSelector
          type="Server"
          selected={params.server}
          onSelect={(server) =>
            setParams({ server, container: params.container })
          }
          disabled={disabled}
        />
        <ContainerSelector
          serverId={params.server}
          selected={params.container}
          onSelect={(container) =>
            setParams({ server: params.server, container })
          }
          disabled={disabled}
        />
      </Group>
    ),
  },
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
    params: { skip_auto_update: false },
    Component: ({ params, setParams, disabled }) => (
      <Group
        style={{ cursor: "pointer" }}
        onClick={() =>
          setParams({ skip_auto_update: !params.skip_auto_update })
        }
      >
        <Switch checked={params.skip_auto_update} disabled={disabled} />
        Skip redeploy
      </Group>
    ),
  },
  RotateAllServerKeys: {
    params: {},
    Component: () => <></>,
  },
  RotateCoreKeys: {
    params: {},
    Component: ({ params, setParams, disabled }) => (
      <Group
        style={{ cursor: !disabled ? "pointer" : undefined }}
        onClick={() => {
          if (!disabled) {
            setParams({ force: !params.force });
          }
        }}
      >
        Force:
        <Switch checked={params.force} disabled={disabled} />
      </Group>
    ),
  },

  SendAlert: {
    params: { message: "" },
    Component: ({ params, setParams, disabled }) => (
      <TextUpdateModal
        title="Alert message"
        value={params.message}
        placeholder="Configure custom alert message"
        onUpdate={(message) => setParams({ message })}
        disabled={disabled}
        useMonaco
        monacoLanguage={undefined}
      />
    ),
  },

  Sleep: {
    params: { duration_ms: 0 },
    Component: ({ params, setParams, disabled }) => {
      const [internal, setInternal] = useState(
        params.duration_ms?.toString() ?? "",
      );
      useEffect(() => {
        setInternal(params.duration_ms?.toString() ?? "");
      }, [params.duration_ms]);
      const durationMs = Number(internal);
      return (
        <TextInput
          placeholder="Duration in milliseconds"
          value={internal}
          onChange={(e) => setInternal(e.target.value)}
          onBlur={() => {
            const duration_ms = Number(internal);
            if (duration_ms) {
              setParams({ duration_ms });
            } else {
              notifications.show({
                message: "Duration must be valid number",
                color: "red",
              });
            }
          }}
          disabled={disabled}
          error={!durationMs ? "Duration must be a valid number" : undefined}
        />
      );
    },
  },
};
