import { usePermissions, useRead } from "@lib/hooks";
import { ReactNode } from "react";
import { Types } from "komodo_client";
import { Section } from "@components/layouts";
import { InspectResponseViewer } from "@components/inspect";

export const StackServiceInspect = ({
  id,
  service,
  useSwarm,
  titleOther,
}: {
  id: string;
  service: string;
  useSwarm: boolean;
  titleOther: ReactNode;
}) => {
  const { specific } = usePermissions({ type: "Stack", id });
  if (!specific.includes(Types.SpecificPermission.Inspect)) {
    return (
      <Section titleOther={titleOther}>
        <div className="min-h-[60vh]">
          <h1>User does not have permission to inspect this Stack service.</h1>
        </div>
      </Section>
    );
  }
  return (
    <Section titleOther={titleOther}>
      <StackServiceInspectInner id={id} service={service} useSwarm={useSwarm} />
    </Section>
  );
};

const StackServiceInspectInner = ({
  id,
  service,
  useSwarm,
}: {
  id: string;
  service: string;
  useSwarm: boolean;
}) => {
  const {
    data: container,
    error,
    isPending,
    isError,
  } = useRead(`InspectStack${useSwarm ? "SwarmService" : "Container"}`, {
    stack: id,
    service,
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
