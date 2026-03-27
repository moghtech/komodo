import { useExecute, useRead } from "@/lib/hooks";
import ConfirmButton from "@/ui/confirm-button";
import ConfirmModalWithDisable from "@/components/confirm-modal-with-disable";
import { Types } from "komodo_client";
import { Pause, Play, RefreshCcw, Square, Trash } from "lucide-react";
import { useNavigate } from "react-router-dom";

const useContainer = (serverId: string, containerName: string) => {
  return useRead("ListDockerContainers", { server: serverId }).data?.find(
    (container) => container.name === containerName,
  );
};

const DestroyContainer = ({
  serverId,
  container: containerName,
}: {
  serverId: string;
  container: string;
}) => {
  const container = useContainer(serverId, containerName);
  const nav = useNavigate();
  const { mutateAsync: destroy, isPending } = useExecute("DestroyContainer", {
    onSuccess: () => nav("/servers/" + serverId),
  });
  const destroying = useRead(
    "GetServerActionState",
    { server: serverId },
    { refetchInterval: 5000 },
  ).data?.pruning_containers;

  if (!container) {
    return null;
  }

  return (
    <ConfirmModalWithDisable
      confirmText={containerName}
      title="Destroy"
      icon={<Trash size="1rem" />}
      onConfirm={() => destroy({ server: serverId, container: containerName })}
      disabled={isPending}
      loading={isPending || destroying}
    >
      Destroy
    </ConfirmModalWithDisable>
  );
};

const RestartContainer = ({
  serverId,
  container: containerName,
}: {
  serverId: string;
  container: string;
}) => {
  const container = useContainer(serverId, containerName);
  const state = container?.state;
  const { mutateAsync: restart, isPending: restartPending } =
    useExecute("RestartContainer");
  const action_state = useRead(
    "GetServerActionState",
    { server: serverId },
    { refetchInterval: 5000 },
  ).data;

  if (!container || state !== Types.ContainerStateStatusEnum.Running) {
    return null;
  }

  return (
    <ConfirmModalWithDisable
      confirmText={containerName}
      icon={<RefreshCcw size="1rem" />}
      onConfirm={() => restart({ server: serverId, container: containerName })}
      disabled={restartPending}
      loading={restartPending || action_state?.restarting_containers}
    >
      Restart
    </ConfirmModalWithDisable>
  );
};

const StartStopContainer = ({
  serverId,
  container: containerName,
}: {
  serverId: string;
  container: string;
}) => {
  const container = useContainer(serverId, containerName);
  const state = container?.state;
  const { mutate: start, isPending: startPending } =
    useExecute("StartContainer");
  const { mutateAsync: stop, isPending: stopPending } = useExecute("StopContainer");
  const action_state = useRead(
    "GetServerActionState",
    { server: serverId },
    { refetchInterval: 5000 },
  ).data;

  if (!container) {
    return null;
  }

  if (state === Types.ContainerStateStatusEnum.Exited) {
    return (
      <ConfirmButton
        icon={<Play size="1rem" />}
        onClick={() => start({ server: serverId, container: containerName })}
        disabled={startPending}
        loading={startPending || action_state?.starting_containers}
      >
        Start
      </ConfirmButton>
    );
  }
  if (state === Types.ContainerStateStatusEnum.Running) {
    return (
      <ConfirmModalWithDisable
        confirmText={containerName}
        icon={<Square size="1rem" />}
        onConfirm={() => stop({ server: serverId, container: containerName })}
        disabled={stopPending}
        loading={stopPending || action_state?.stopping_containers}
      >
        Stop
      </ConfirmModalWithDisable>
    );
  }
};

const PauseUnpauseContainer = ({
  serverId,
  container: containerName,
}: {
  serverId: string;
  container: string;
}) => {
  const container = useContainer(serverId, containerName);
  const state = container?.state;
  const { mutate: unpause, isPending: unpausePending } =
    useExecute("UnpauseContainer");
  const { mutateAsync: pause, isPending: pausePending } =
    useExecute("PauseContainer");
  const action_state = useRead(
    "GetServerActionState",
    { server: serverId },
    { refetchInterval: 5000 },
  ).data;

  if (!container) {
    return null;
  }

  if (state === Types.ContainerStateStatusEnum.Paused) {
    return (
      <ConfirmButton
        icon={<Play size="1rem" />}
        onClick={() => unpause({ server: serverId, container: containerName })}
        disabled={unpausePending}
        loading={unpausePending || action_state?.unpausing_containers}
      >
        Unpause
      </ConfirmButton>
    );
  }
  if (state === Types.ContainerStateStatusEnum.Running) {
    return (
      <ConfirmModalWithDisable
        confirmText={containerName}
        icon={<Pause size="1rem" />}
        onConfirm={() => pause({ server: serverId, container: containerName })}
        disabled={pausePending}
        loading={pausePending || action_state?.pausing_containers}
      >
        Pause
      </ConfirmModalWithDisable>
    );
  }
};

type IdContainerComponent = React.FC<{ serverId: string; container: string }>;

export const ContainerExecutions: { [action: string]: IdContainerComponent } = {
  RestartContainer,
  PauseUnpauseContainer,
  StartStopContainer,
  DestroyContainer,
};
