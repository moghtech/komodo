import { usePermissions, useRead } from "@lib/hooks";
import { ReactNode } from "react";
import { Types } from "komodo_client";
import { Section } from "@components/layouts";
import { InspectResponseViewer } from "@components/inspect";

export const DeploymentInspect = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) => {
  const { specific } = usePermissions({ type: "Deployment", id });
  if (!specific.includes(Types.SpecificPermission.Inspect)) {
    return (
      <Section titleOther={titleOther}>
        <div className="min-h-[60vh]">
          <h1>User does not have permission to inspect this Deployment.</h1>
        </div>
      </Section>
    );
  }
  return (
    <Section titleOther={titleOther}>
      <DeploymentInspectInner id={id} />
    </Section>
  );
};

const DeploymentInspectInner = ({ id }: { id: string }) => {
  const {
    data: container,
    error,
    isPending,
    isError,
  } = useRead("InspectDeploymentContainer", {
    deployment: id,
  });
  return (
    <InspectResponseViewer
      response={container}
      error={error}
      isPending={isPending}
      isError={isError}
    />
  );
};
