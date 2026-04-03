import { useExecute, useRead } from "@/lib/hooks";
import { Types } from "komodo_client";
import { useStack } from ".";
import ConfirmButton from "@/ui/confirm-button";
import { ICONS } from "@/lib/icons";
import ConfirmModalWithDisable from "@/components/confirm-modal-with-disable";

export const DeployStack = ({
  id,
  service,
}: {
  id: string;
  service?: string;
}) => {
  const stack = useStack(id);
  const state = stack?.info.state;
  const { mutateAsync: deploy, isPending } = useExecute("DeployStack");
  const deploying = useRead(
    "GetStackActionState",
    { stack: id },
    { refetchInterval: 5000 },
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
      <ConfirmModalWithDisable
        confirmText={`${stack?.name}${service ? ` - ${service}` : ""}`}
        icon={<ICONS.Deploy size="1rem" />}
        onConfirm={() =>
          deploy({ stack: id, services: service ? [service] : [] })
        }
        disabled={isPending}
        loading={isPending || deploying}
      >
        Deploy
      </ConfirmModalWithDisable>
    );
  }

  return (
    <ConfirmButton
      icon={<ICONS.Deploy size="1rem" />}
      onClick={() => deploy({ stack: id, services: service ? [service] : [] })}
      disabled={isPending}
      loading={isPending || deploying}
    >
      Deploy
    </ConfirmButton>
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
  const { mutateAsync: destroy, isPending } = useExecute("DestroyStack");
  const destroying = useRead(
    "GetStackActionState",
    { stack: id },
    { refetchInterval: 5000 },
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
    <ConfirmModalWithDisable
      confirmText={`${stack?.name}${service ? ` - ${service}` : ""}`}
      icon={<ICONS.Destroy size="1rem" />}
      onConfirm={() =>
        destroy({ stack: id, services: service ? [service] : [] })
      }
      disabled={isPending}
      loading={isPending || destroying}
    >
      Destroy
    </ConfirmModalWithDisable>
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
  const actionState = useRead(
    "GetStackActionState",
    { stack: id },
    { refetchInterval: 5000 },
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
      icon={<ICONS.Pull size="1rem" />}
      onClick={() => pull({ stack: id, services: service ? [service] : [] })}
      disabled={pullPending}
      loading={pullPending || actionState?.pulling}
    >
      {`Pull Image${service ? "" : "s"}`}
    </ConfirmButton>
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
  const { mutateAsync: restart, isPending: restartPending } =
    useExecute("RestartStack");
  const actionState = useRead(
    "GetStackActionState",
    { stack: id },
    { refetchInterval: 5000 },
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
    <ConfirmModalWithDisable
      confirmText={`${stack?.name}${service ? ` - ${service}` : ""}`}
      icon={<ICONS.Restart size="1rem" />}
      onConfirm={() =>
        restart({ stack: id, services: service ? [service] : [] })
      }
      disabled={restartPending}
      loading={restartPending || actionState?.restarting}
    >
      Restart
    </ConfirmModalWithDisable>
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
  const { mutateAsync: stop, isPending: stopPending } = useExecute("StopStack");
  const actionState = useRead(
    "GetStackActionState",
    { stack: id },
    { refetchInterval: 5000 },
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
          icon={<ICONS.Start size="1rem" />}
          onClick={() =>
            start({ stack: id, services: service ? [service] : [] })
          }
          disabled={startPending}
          loading={startPending || actionState?.starting}
        >
          Start
        </ConfirmButton>
      )}
      {showStop && (
        <ConfirmModalWithDisable
          confirmText={`${stack?.name}${service ? ` - ${service}` : ""}`}
          icon={<ICONS.Stop size="1rem" />}
          onConfirm={() =>
            stop({ stack: id, services: service ? [service] : [] })
          }
          disabled={stopPending}
          loading={stopPending || actionState?.stopping}
        >
          Stop
        </ConfirmModalWithDisable>
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
  const { mutateAsync: pause, isPending: pausePending } =
    useExecute("PauseStack");
  const actionState = useRead(
    "GetStackActionState",
    { stack: id },
    { refetchInterval: 5000 },
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
        icon={<ICONS.Start size="1rem" />}
        onClick={() =>
          unpause({ stack: id, services: service ? [service] : [] })
        }
        disabled={unpausePending}
        loading={unpausePending || actionState?.unpausing}
      >
        Unpause
      </ConfirmButton>
    );
  }
  if (
    (service && container_state === Types.ContainerStateStatusEnum.Running) ||
    state === Types.StackState.Running
  ) {
    return (
      <ConfirmModalWithDisable
        confirmText={`${stack?.name}${service ? ` - ${service}` : ""}`}
        icon={<ICONS.Pause size="1rem" />}
        onConfirm={() =>
          pause({ stack: id, services: service ? [service] : [] })
        }
        disabled={pausePending}
        loading={pausePending || actionState?.pausing}
      >
        Pause
      </ConfirmModalWithDisable>
    );
  }
};
