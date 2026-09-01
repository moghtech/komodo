import { useEffect, useState } from "react";
import { Group, Select, Stack, Text } from "@mantine/core";
import { useExecute, useInvalidate, useListItem, useRead } from "@/lib/hooks";
import { Types } from "komodo_client";
import { EXECUTION_ACTION_STATE_REQUERY_MS, parseKeyValue } from "@/lib/utils";
import { useDeployment } from ".";
import { ICONS } from "@/lib/icons";
import { ConfirmButton } from "mogh_ui";
import ConfirmModalWithDisable from "@/components/confirm-modal-with-disable";

interface DeploymentId {
  id: string;
}

const useInvalidateActionState = () => {
  const invalidate = useInvalidate();
  return {
    onSuccess: () =>
      setTimeout(
        () => invalidate(["GetDeploymentActionState"]),
        EXECUTION_ACTION_STATE_REQUERY_MS,
      ),
  };
};

export function DeployDeployment({ id }: DeploymentId) {
  const deployment = useRead("GetDeployment", { deployment: id }).data;
  const [signal, setSignal] = useState<Types.TerminationSignal>();

  useEffect(
    () => setSignal(deployment?.config?.termination_signal),
    [deployment?.config?.termination_signal],
  );

  const { mutateAsync: deploy, isPending } = useExecute(
    "Deploy",
    useInvalidateActionState(),
  );

  const deployment_item = useListItem("Deployment", id);

  const deploying = useRead(
    "GetDeploymentActionState",
    { deployment: id },
    { refetchInterval: 5_000 },
  ).data?.deploying;

  const pending = isPending || deploying;

  if (!deployment) return null;

  const deployed =
    deployment_item?.info.state !== Types.DeploymentState.NotDeployed &&
    deployment_item?.info.state !== Types.DeploymentState.Unknown;

  const term_signal_labels =
    deployed &&
    parseKeyValue(deployment.config?.term_signal_labels ?? "").map(
      (s) =>
        ({ signal: s.key, label: s.value }) as Types.TerminationSignalLabel,
    );

  if (deployed) {
    return (
      <ConfirmModalWithDisable
        confirmText={deployment.name}
        icon={<ICONS.Deploy size="1rem" />}
        onConfirm={() => deploy({ deployment: id, stop_signal: signal })}
        disabled={pending}
        loading={pending}
        additional={
          term_signal_labels && term_signal_labels.length > 1 ? (
            <TermSignalSelector
              signals={term_signal_labels}
              signal={signal}
              setSignal={setSignal}
            />
          ) : undefined
        }
      >
        Redeploy
      </ConfirmModalWithDisable>
    );
  } else {
    return (
      <ConfirmButton
        icon={<ICONS.Deploy size="1rem" />}
        onClick={() => deploy({ deployment: id })}
        disabled={pending}
        loading={pending}
      >
        Deploy
      </ConfirmButton>
    );
  }
}

export function DestroyDeployment({ id }: DeploymentId) {
  const deployment = useRead("GetDeployment", { deployment: id }).data;
  const [signal, setSignal] = useState<Types.TerminationSignal>();

  useEffect(
    () => setSignal(deployment?.config?.termination_signal),
    [deployment?.config?.termination_signal],
  );

  const { mutateAsync: destroy, isPending } = useExecute(
    "DestroyDeployment",
    useInvalidateActionState(),
  );

  const state = useListItem("Deployment", id)?.info.state;

  const destroying = useRead(
    "GetDeploymentActionState",
    {
      deployment: id,
    },
    { refetchInterval: 5_000 },
  ).data?.destroying;

  const pending = isPending || destroying;

  if (!deployment) return null;
  if (state === Types.DeploymentState.NotDeployed) return null;

  const term_signal_labels = parseKeyValue(
    deployment.config?.term_signal_labels ?? "",
  ).map(
    (s) => ({ signal: s.key, label: s.value }) as Types.TerminationSignalLabel,
  );

  return (
    <ConfirmModalWithDisable
      confirmText={deployment.name}
      icon={<ICONS.Destroy size="1rem" />}
      onConfirm={() => destroy({ deployment: id, signal })}
      disabled={pending}
      loading={pending}
      additional={
        term_signal_labels && term_signal_labels.length > 1 ? (
          <TermSignalSelector
            signals={term_signal_labels}
            signal={signal}
            setSignal={setSignal}
          />
        ) : undefined
      }
    >
      Destroy
    </ConfirmModalWithDisable>
  );
}

export function PullDeployment({ id }: DeploymentId) {
  const deployment = useDeployment(id);
  const { mutate: pull, isPending: pullPending } = useExecute(
    "PullDeployment",
    useInvalidateActionState(),
  );
  const action_state = useRead(
    "GetDeploymentActionState",
    {
      deployment: id,
    },
    { refetchInterval: 5_000 },
  ).data;

  if (!deployment || deployment.info.swarm_id) return null;

  return (
    <ConfirmButton
      icon={<ICONS.Pull size="1rem" />}
      onClick={() => pull({ deployment: id })}
      disabled={pullPending}
      loading={pullPending || action_state?.pulling}
    >
      Pull Image
    </ConfirmButton>
  );
}

export function RestartDeployment({ id }: DeploymentId) {
  const deployment = useDeployment(id);
  const state = deployment?.info.state;
  const { mutateAsync: restart, isPending: restartPending } = useExecute(
    "RestartDeployment",
    useInvalidateActionState(),
  );
  const action_state = useRead(
    "GetDeploymentActionState",
    {
      deployment: id,
    },
    { refetchInterval: 5_000 },
  ).data;

  if (!deployment || deployment.info.swarm_id) return null;

  if (state !== Types.DeploymentState.Running) {
    return null;
  }

  return (
    <ConfirmModalWithDisable
      confirmText={deployment.name}
      icon={<ICONS.Refresh size="1rem" />}
      onConfirm={() => restart({ deployment: id })}
      disabled={restartPending}
      loading={restartPending || action_state?.restarting}
    >
      Restart
    </ConfirmModalWithDisable>
  );
}

export function StartStopDeployment({ id }: DeploymentId) {
  const deployment = useDeployment(id);
  const state = deployment?.info.state;
  const { mutate: start, isPending: startPending } = useExecute(
    "StartDeployment",
    useInvalidateActionState(),
  );
  const action_state = useRead(
    "GetDeploymentActionState",
    {
      deployment: id,
    },
    { refetchInterval: 5_000 },
  ).data;

  if (!deployment || deployment.info.swarm_id) return null;

  if (state === Types.DeploymentState.Exited) {
    return (
      <ConfirmButton
        icon={<ICONS.Start size="1rem" />}
        onClick={() => start({ deployment: id })}
        disabled={startPending}
        loading={startPending || action_state?.starting}
      >
        Start
      </ConfirmButton>
    );
  }
  if (state !== Types.DeploymentState.NotDeployed) {
    return <StopDeployment id={id} />;
  }
}

function StopDeployment({ id }: DeploymentId) {
  const deployment = useRead("GetDeployment", { deployment: id }).data;
  const [signal, setSignal] = useState<Types.TerminationSignal>();

  useEffect(
    () => setSignal(deployment?.config?.termination_signal),
    [deployment?.config?.termination_signal],
  );

  const { mutateAsync: stop, isPending } = useExecute(
    "StopDeployment",
    useInvalidateActionState(),
  );
  const stopping = useRead(
    "GetDeploymentActionState",
    {
      deployment: id,
    },
    { refetchInterval: 5_000 },
  ).data?.stopping;

  const pending = isPending || stopping;

  if (!deployment) return null;

  const term_signal_labels = parseKeyValue(
    deployment.config?.term_signal_labels ?? "",
  ).map(
    (s) => ({ signal: s.key, label: s.value }) as Types.TerminationSignalLabel,
  );

  return (
    <ConfirmModalWithDisable
      confirmText={deployment.name}
      icon={<ICONS.Stop size="1rem" />}
      onConfirm={() => stop({ deployment: id, signal })}
      disabled={pending}
      loading={pending}
      additional={
        term_signal_labels && term_signal_labels.length > 1 ? (
          <TermSignalSelector
            signals={term_signal_labels}
            signal={signal}
            setSignal={setSignal}
          />
        ) : undefined
      }
    >
      Stop
    </ConfirmModalWithDisable>
  );
}

function TermSignalSelector({
  signals,
  signal,
  setSignal,
}: {
  signals: Types.TerminationSignalLabel[];
  signal: Types.TerminationSignal | undefined;
  setSignal: (signal: Types.TerminationSignal) => void;
}) {
  const label = signals.find((s) => s.signal === signal)?.label;
  return (
    <Stack>
      <Group justify="flex-end">
        <Text c="dimmed">Termination</Text>
      </Group>
      <Group justify="flex-end">
        <Text c="dimmed">{label}</Text>
        <Select
          value={signal}
          onChange={(signal) =>
            signal && setSignal(signal as Types.TerminationSignal)
          }
          data={signals.map((s) => s.signal)}
        />
      </Group>
    </Stack>
  );
}

export function PauseUnpauseDeployment({ id }: DeploymentId) {
  const deployment = useDeployment(id);
  const state = deployment?.info.state;
  const { mutate: unpause, isPending: unpausePending } = useExecute(
    "UnpauseDeployment",
    useInvalidateActionState(),
  );
  const { mutateAsync: pause, isPending: pausePending } = useExecute(
    "PauseDeployment",
    useInvalidateActionState(),
  );
  const action_state = useRead(
    "GetDeploymentActionState",
    {
      deployment: id,
    },
    { refetchInterval: 5_000 },
  ).data;

  if (!deployment || deployment.info.swarm_id) return null;

  if (state === Types.DeploymentState.Paused) {
    return (
      <ConfirmButton
        icon={<ICONS.Start size="1rem" />}
        onClick={() => unpause({ deployment: id })}
        disabled={unpausePending}
        loading={unpausePending || action_state?.unpausing}
      >
        Unpause
      </ConfirmButton>
    );
  }
  if (state === Types.DeploymentState.Running) {
    return (
      <ConfirmModalWithDisable
        confirmText={deployment.name}
        icon={<ICONS.Pause size="1rem" />}
        onConfirm={() => pause({ deployment: id })}
        disabled={pausePending}
        loading={pausePending || action_state?.pausing}
      >
        Pause
      </ConfirmModalWithDisable>
    );
  }
}
