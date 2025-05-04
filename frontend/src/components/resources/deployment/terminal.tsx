import { ReactNode } from "react";
import { ContainerTerminal } from "@components/terminal";
import { Types } from "komodo_client";

export const DeploymentTerminal = ({
  deployment,
  titleOther,
}: {
  deployment: Types.DeploymentListItem;
  titleOther?: ReactNode;
}) => {
  return (
    deployment.info.server_id && (
      <ContainerTerminal
        titleOther={titleOther}
        server={deployment.info.server_id}
        container={deployment.name}
      />
    )
  );
};
