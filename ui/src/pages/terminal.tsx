import DockerResourceLink from "@/components/docker/link";
import TargetTerminal from "@/components/terminal/target";
import { useSetTitle, useWrite } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { useDeployment } from "@/resources/deployment";
import { useServer } from "@/resources/server";
import { useStack } from "@/resources/stack";
import ConfirmButton from "@/ui/confirm-button";
import Page from "@/ui/page";
import { Group, Text } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { Types } from "komodo_client";
import { ReactNode, useMemo } from "react";
import { useNavigate, useParams } from "react-router-dom";
import ResourceLink from "@/resources/link";

type WithTerminal = "servers" | "deployments" | "stacks" | string;

export default function Terminal() {
  const { type, id, terminal, container, service } = useParams() as {
    type: WithTerminal;
    id: string;
    terminal: string;
    container: string | undefined;
    service: string | undefined;
  };
  switch (type) {
    case "servers":
      if (container) {
        return (
          <ContainerTerminalPage
            type={type as WithTerminal}
            id={id}
            container={container}
            terminal={terminal}
          />
        );
      } else {
        return (
          <ServerTerminalPage
            type={type as WithTerminal}
            id={id}
            terminal={terminal}
          />
        );
      }

    case "stacks":
      return service ? (
        <StackServiceTerminalPage
          type={type as WithTerminal}
          id={id}
          service={service}
          terminal={terminal}
        />
      ) : (
        <Text>Missing :service in URL</Text>
      );

    case "deployments":
      return (
        <DeploymentTerminalPage
          type={type as WithTerminal}
          id={id}
          terminal={terminal}
        />
      );

    default:
      return <Text>This resource type does not have any Terminals.</Text>;
  }
}

function ServerTerminalPage({
  type: _type,
  id,
  terminal,
}: {
  type: WithTerminal;
  id: string;
  terminal: string;
}) {
  const server = useServer(id);
  useSetTitle(`${server?.name} | Terminal | ${terminal}`);
  const target: Types.TerminalTarget = useMemo(
    () => ({
      type: "Server",
      params: { server: id },
    }),
    [id],
  );
  return (
    <TerminalLayout
      terminal={terminal}
      target={target}
      Link={<ResourceLink type="Server" id={id} />}
    />
  );
}

function ContainerTerminalPage({
  type: _type,
  id,
  container,
  terminal,
}: {
  type: WithTerminal;
  id: string;
  container: string;
  terminal: string;
}) {
  const server = useServer(id);
  useSetTitle(`${server?.name} | ${container} Terminal | ${terminal}`);
  const target: Types.TerminalTarget = useMemo(
    () => ({
      type: "Container",
      params: { server: id, container },
    }),
    [id, container],
  );
  return (
    <TerminalLayout
      terminal={terminal}
      target={target}
      Link={
        <DockerResourceLink type="Container" serverId={id} name={container} />
      }
    />
  );
}

function StackServiceTerminalPage({
  type: _type,
  id,
  service,
  terminal,
}: {
  type: WithTerminal;
  id: string;
  service: string;
  terminal: string;
}) {
  const stack = useStack(id);
  useSetTitle(`${stack?.name} | ${service} Terminal | ${terminal}`);
  const target: Types.TerminalTarget = useMemo(
    () => ({
      type: "Stack",
      params: { stack: id, service },
    }),
    [id, service],
  );
  return (
    <TerminalLayout
      terminal={terminal}
      target={target}
      Link={
        <Group>
          <ResourceLink type="Stack" id={target.params.stack} />
          {/* {target.params.service && (
            <StackServiceLink
              id={target.params.stack}
              service={target.params.service}
            />
          )} */}
        </Group>
      }
    />
  );
}

function DeploymentTerminalPage({
  type: _type,
  id,
  terminal,
}: {
  type: WithTerminal;
  id: string;
  terminal: string;
}) {
  const deployment = useDeployment(id);
  useSetTitle(`${deployment?.name} | Terminal | ${terminal}`);
  const target: Types.TerminalTarget = useMemo(
    () => ({
      type: "Deployment",
      params: { deployment: id },
    }),
    [id],
  );
  return (
    <TerminalLayout
      terminal={terminal}
      target={target}
      Link={<ResourceLink type="Deployment" id={id} />}
    />
  );
}

function TerminalLayout({
  terminal,
  target,
  Link,
}: {
  terminal: string;
  target: Types.TerminalTarget;
  Link: ReactNode;
}) {
  return (
    <Page
      title={terminal}
      icon={ICONS.Terminal}
      customDescription={
        <>
          <Text>Terminal</Text>|{Link}|
          <DeleteTerminal terminal={terminal} target={target} />
        </>
      }
    >
      <TargetTerminal terminal={terminal} target={target} selected _reconnect />
    </Page>
  );
}

function DeleteTerminal({
  terminal,
  target,
}: {
  terminal: string;
  target: Types.TerminalTarget;
}) {
  const nav = useNavigate();
  const { mutate, isPending } = useWrite("DeleteTerminal", {
    onSuccess: () => {
      notifications.show({ message: `Deleted Terminal '${terminal}'` });
      nav("/terminals");
    },
  });
  return (
    <ConfirmButton
      variant="filled"
      color="red"
      icon={<ICONS.Delete size="1rem" />}
      onClick={() =>
        mutate({
          target,
          terminal,
        })
      }
      loading={isPending}
      w={120}
      size="xs"
    >
      Delete
    </ConfirmButton>
  );
}
