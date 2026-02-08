import { MonacoEditor } from "@/components/monaco";
import StackServiceSelector from "@/components/stack-service-selector";
import { textToEnv } from "@/lib/utils";
import ResourceSelector from "@/resources/selector";
import EnableSwitch from "@/ui/enable-switch";
import TextUpdateModal from "@/ui/text-update-modal";
import {
  Button,
  Group,
  Modal,
  SimpleGrid,
  Stack,
  Switch,
  Text,
  TextInput,
} from "@mantine/core";
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
        monacoLanguage="string_list"
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
      <TextUpdateModal
        title="Match stacks"
        value={
          params.pattern ||
          "# Match stacks by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
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
        monacoLanguage="string_list"
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
      <TextUpdateModal
        title="Match stacks"
        value={
          params.pattern ||
          "# Match stacks by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
        monacoLanguage="string_list"
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
      <TextUpdateModal
        title="Match stacks"
        value={
          params.pattern ||
          "# Match stacks by name, id, wildcard, or \\regex\\.\n"
        }
        onUpdate={(pattern) => setParams({ pattern })}
        disabled={disabled}
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
      const [open, setOpen] = useState(false);
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
        setOpen(false);
      };

      return (
        <>
          <Button disabled={disabled}>Configure</Button>

          <Modal
            opened={open}
            onClose={() => setOpen(false)}
            title="Run Stack Service"
            size="lg"
          >
            <Stack>
              <SimpleGrid cols={{ base: 1, md: 2 }}>
                <Group>
                  <Text c="dimmed">Stack</Text>
                  <ResourceSelector
                    type="Stack"
                    selected={stack}
                    onSelect={(id) => setStack(id)}
                    disabled={disabled}
                  />
                </Group>
                <Group>
                  <Text c="dimmed">Service</Text>
                  <StackServiceSelector
                    stackId={stack}
                    selected={service}
                    onSelect={setService}
                    disabled={disabled}
                  />
                </Group>
              </SimpleGrid>

              <Group>
                <Text c="dimmed">Command</Text>
                <TextInput
                  value={commandText}
                  onChange={(e) => setCommand(e.target.value)}
                  disabled={disabled}
                />
              </Group>

              <SimpleGrid cols={{ base: 2, md: 4 }}>
                <EnableSwitch
                  label="No TTY"
                  checked={no_tty}
                  onCheckedChange={setNoTty}
                  disabled={disabled}
                />
                <EnableSwitch
                  label="No Dependencies"
                  checked={no_deps}
                  onCheckedChange={setNoDeps}
                  disabled={disabled}
                />
                <EnableSwitch
                  label="Detach"
                  checked={detach}
                  onCheckedChange={setDetach}
                  disabled={disabled}
                />
                <EnableSwitch
                  label="Service Ports"
                  checked={service_ports}
                  onCheckedChange={setServicePorts}
                  disabled={disabled}
                />
                <EnableSwitch
                  label="Pull Image"
                  checked={pull}
                  onCheckedChange={setPull}
                  disabled={disabled}
                />
              </SimpleGrid>
            </Stack>

            <SimpleGrid cols={{ base: 1, md: 2 }}>
              <Group>
                <Text c="dimmed">Working Directory</Text>
                <TextInput
                  placeholder="/work/dir"
                  value={workdir}
                  onChange={(e) => setWorkdir(e.target.value)}
                  disabled={disabled}
                />
              </Group>
              <Group>
                <Text c="dimmed">User</Text>
                <TextInput
                  placeholder="uid:gid or user"
                  value={user}
                  onChange={(e) => setUser(e.target.value)}
                  disabled={disabled}
                />
              </Group>
              <Group>
                <Text c="dimmed">Entrypoint</Text>
                <TextInput
                  value={entrypoint}
                  onChange={(e) => setEntrypoint(e.target.value)}
                  disabled={disabled}
                />
              </Group>
            </SimpleGrid>

            <Group>
              <Text c="dimmed">Extra Environment Variables</Text>
              <MonacoEditor
                value={envText}
                onValueChange={setEnvText}
                language="key_value"
                readOnly={disabled}
              />
            </Group>

            {!disabled && (
              <Button onClick={onConfirm} leftSection={<CheckCircle />}>
                Confirm
              </Button>
            )}
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
