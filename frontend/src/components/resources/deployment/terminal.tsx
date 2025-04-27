import { ReactNode } from "react";
import { useDeployment } from ".";
import { ContainerTerminal } from "@components/terminal";

export const DeploymentTerminal = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther?: ReactNode;
}) => {
  const deployment = useDeployment(id);

  return (
    deployment &&
    deployment.info.server_id && (
      <ContainerTerminal
        titleOther={titleOther}
        server={deployment.info.server_id}
        container_name={deployment.name}
      />
    )
  );
};
