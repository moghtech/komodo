import { ResourceLink } from "@/resources/common";
import { Types } from "komodo_client";
import DockerResourceLink from "@/components/docker/link";
import { Group } from "@mantine/core";

export default function TerminalTargetLink({ target }: { target: Types.TerminalTarget }) {
  switch (target.type) {
    case "Server":
      return <ResourceLink type="Server" id={target.params.server!} />;
    case "Container":
      return (
        <DockerResourceLink
          type="container"
          serverId={target.params.server}
          name={target.params.container}
        />
      );
    case "Stack":
      return (
        <Group>
          <ResourceLink type="Stack" id={target.params.stack} />
          {/* {target.params.service && (
            <StackServiceLink
              id={target.params.stack}
              service={target.params.service}
            />
          )} */}
        </Group>
      );
    case "Deployment":
      return <ResourceLink type="Deployment" id={target.params.deployment} />;
  }
}
