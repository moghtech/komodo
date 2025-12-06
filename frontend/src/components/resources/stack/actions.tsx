import { ActionWithDialog, ConfirmButton } from "@components/util";
import { useExecute, useRead } from "@lib/hooks";
import {
  Download,
  Pause,
  Play,
  RefreshCcw,
  Rocket,
  Square,
  Trash,
} from "lucide-react";
import { useStack } from ".";
import { Types } from "komodo_client";

export const DeployStack = ({
  id,
  service,
}: {
  id: string;
  service?: string;
}) => {
  const stack = useStack(id);
  const state = stack?.info.state;
  const { mutate: deploy, isPending } = useExecute("DeployStack");
  const deploying = useRead(
    "GetStackActionState",
    { stack: id },
    { refetchInterval: 5000 }
  ).data?.deploying;
  const services = useRead("ListStackServices", { stack: id }).data;
  const container_state =
    (service
      ? services?.find((s) => s.service === service)?.container?.state
      : undefined) ?? Types.ContainerStateStatusEnum.Empty;

  if (!stack || state === Types.StackState.Unknown) {
    return null;
  }
  const deployed =
    state !== undefined &&
    (service !== undefined
      ? container_state !== Types.ContainerStateStatusEnum.Empty
      : [
          Types.StackState.Running,
          Types.StackState.Paused,
          Types.StackState.Stopped,
          Types.StackState.Restarting,
          Types.StackState.Unhealthy,
        ].includes(state));

  if (deployed) {
    return (
      <ActionWithDialog
        name={`${stack?.name}${service ? ` - ${service}` : ""}`}
        title="Redeploy"
        icon={<Rocket className="h-4 w-4" />}
        onClick={() =>
          deploy({ stack: id, services: service ? [service] : [] })
        }
        disabled={isPending}
        loading={isPending || deploying}
      />
    );
  }

  return (
    <ConfirmButton
      title="Deploy"
      icon={<Rocket className="w-4 h-4" />}
      onClick={() => deploy({ stack: id, services: service ? [service] : [] })}
      disabled={isPending}
      loading={isPending || deploying}
    />
  );
};

export const DestroyStack = ({
  id,
  service,
}: {
  id: string;
  service?: string;
}) => {
  const stack = useStack(id);
  const state = stack?.info.state;
  const { mutate: destroy, isPending } = useExecute("DestroyStack");
  const destroying = useRead(
    "GetStackActionState",
    { stack: id },
    { refetchInterval: 5000 }
  ).data?.destroying;
  const services = useRead("ListStackServices", { stack: id }).data;
  const container_state =
    (service
      ? services?.find((s) => s.service === service)?.container?.state
      : undefined) ?? Types.ContainerStateStatusEnum.Empty;

  if (
    !stack || service !== undefined
      ? container_state === Types.ContainerStateStatusEnum.Empty
      : state === undefined ||
        [Types.StackState.Unknown, Types.StackState.Down].includes(state!)
  ) {
    return null;
  }

  return (
    <ActionWithDialog
      name={`${stack?.name}${service ? ` - ${service}` : ""}`}
      title="Destroy"
      icon={<Trash className="h-4 w-4" />}
      onClick={() => destroy({ stack: id, services: service ? [service] : [] })}
      disabled={isPending}
      loading={isPending || destroying}
    />
  );
};

export const PullStack = ({
  id,
  service,
}: {
  id: string;
  service?: string;
}) => {
  const stack = useStack(id);
  const { mutate: pull, isPending: pullPending } = useExecute("PullStack");
  const action_state = useRead(
    "GetStackActionState",
    { stack: id },
    { refetchInterval: 5000 }
  ).data;

  if (
    !stack ||
    stack.info.swarm_id ||
    (stack?.info.missing_files.length ?? 0) > 0
  ) {
    return null;
  }

  return (
    <ConfirmButton
      title={`Pull Image${service ? "" : "s"}`}
      icon={<Download className="h-4 w-4" />}
      onClick={() => pull({ stack: id, services: service ? [service] : [] })}
      disabled={pullPending}
      loading={pullPending || action_state?.pulling}
    />
  );
};

export const RestartStack = ({
  id,
  service,
}: {
  id: string;
  service?: string;
}) => {
  const stack = useStack(id);
  const state = stack?.info.state;
  const { mutate: restart, isPending: restartPending } =
    useExecute("RestartStack");
  const action_state = useRead(
    "GetStackActionState",
    { stack: id },
    { refetchInterval: 5000 }
  ).data;
  const services = useRead("ListStackServices", { stack: id }).data;
  const container_state =
    (service
      ? services?.find((s) => s.service === service)?.container?.state
      : undefined) ?? Types.ContainerStateStatusEnum.Empty;

  if (
    !stack ||
    stack.info.swarm_id ||
    stack?.info.project_missing ||
    (service && container_state !== Types.ContainerStateStatusEnum.Running) ||
    // Only show if running or unhealthy
    (state !== Types.StackState.Running && state !== Types.StackState.Unhealthy)
  ) {
    return null;
  }

  return (
    <ActionWithDialog
      name={`${stack?.name}${service ? ` - ${service}` : ""}`}
      title="Restart"
      icon={<RefreshCcw className="h-4 w-4" />}
      onClick={() => restart({ stack: id, services: service ? [service] : [] })}
      disabled={restartPending}
      loading={restartPending || action_state?.restarting}
    />
  );
};

export const StartStopStack = ({
  id,
  service,
}: {
  id: string;
  service?: string;
}) => {
  const stack = useStack(id);
  const state = stack?.info.state ?? Types.StackState.Unknown;
  const { mutate: start, isPending: startPending } = useExecute("StartStack");
  const { mutate: stop, isPending: stopPending } = useExecute("StopStack");
  const action_state = useRead(
    "GetStackActionState",
    { stack: id },
    { refetchInterval: 5000 }
  ).data;
  const services = useRead("ListStackServices", { stack: id }).data;
  const container_state =
    (service
      ? services?.find((s) => s.service === service)?.container?.state
      : undefined) ?? Types.ContainerStateStatusEnum.Empty;

  if (
    !stack ||
    stack.info.swarm_id ||
    [Types.StackState.Down, Types.StackState.Unknown].includes(state)
  ) {
    return null;
  }

  const showStart = service
    ? ((container_state &&
        container_state !== Types.ContainerStateStatusEnum.Running) ??
      false)
    : state !== Types.StackState.Running;
  const showStop = service
    ? ((container_state &&
        container_state !== Types.ContainerStateStatusEnum.Exited) ??
      false)
    : state !== Types.StackState.Stopped;

  return (
    <>
      {showStart && (
        <ConfirmButton
          title="Start"
          icon={<Play className="h-4 w-4" />}
          onClick={() =>
            start({ stack: id, services: service ? [service] : [] })
          }
          disabled={startPending}
          loading={startPending || action_state?.starting}
        />
      )}
      {showStop && (
        <ActionWithDialog
          name={`${stack?.name}${service ? ` - ${service}` : ""}`}
          title="Stop"
          icon={<Square className="h-4 w-4" />}
          onClick={() =>
            stop({ stack: id, services: service ? [service] : [] })
          }
          disabled={stopPending}
          loading={stopPending || action_state?.stopping}
        />
      )}
    </>
  );
};

export const PauseUnpauseStack = ({
  id,
  service,
}: {
  id: string;
  service?: string;
}) => {
  const stack = useStack(id);
  const state = stack?.info.state;
  const { mutate: unpause, isPending: unpausePending } =
    useExecute("UnpauseStack");
  const { mutate: pause, isPending: pausePending } = useExecute("PauseStack");
  const action_state = useRead(
    "GetStackActionState",
    { stack: id },
    { refetchInterval: 5000 }
  ).data;
  const services = useRead("ListStackServices", { stack: id }).data;
  const container_state =
    (service
      ? services?.find((s) => s.service === service)?.container?.state
      : undefined) ?? Types.ContainerStateStatusEnum.Empty;

  if (!stack || stack.info.swarm_id || stack?.info.project_missing) {
    return null;
  }

  if (
    (service && container_state === Types.ContainerStateStatusEnum.Paused) ||
    state === Types.StackState.Paused
  ) {
    return (
      <ConfirmButton
        title="Unpause"
        icon={<Play className="h-4 w-4" />}
        onClick={() =>
          unpause({ stack: id, services: service ? [service] : [] })
        }
        disabled={unpausePending}
        loading={unpausePending || action_state?.unpausing}
      />
    );
  }
  if (
    (service && container_state === Types.ContainerStateStatusEnum.Running) ||
    state === Types.StackState.Running
  ) {
    return (
      <ActionWithDialog
        name={`${stack?.name}${service ? ` - ${service}` : ""}`}
        title="Pause"
        icon={<Pause className="h-4 w-4" />}
        onClick={() => pause({ stack: id, services: service ? [service] : [] })}
        disabled={pausePending}
        loading={pausePending || action_state?.pausing}
      />
    );
  }
};
