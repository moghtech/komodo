import { usePermissions, useRead } from "@lib/hooks";
import { ReactNode } from "react";
import { Types } from "komodo_client";
import { Section } from "@components/layouts";
import { InspectContainerView } from "@components/inspect";

export const StackServiceInspect = ({
  id,
  service,
  titleOther,
}: {
  id: string;
  service: string;
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
      <StackServiceInspectInner id={id} service={service} />
    </Section>
  );
};

const StackServiceInspectInner = ({
  id,
  service,
}: {
  id: string;
  service: string;
}) => {
  const {
    data: container,
    error,
    isPending,
    isError,
  } = useRead("InspectStackContainer", {
    stack: id,
    service,
  });
  return (
    <InspectContainerView
      container={container}
      error={error}
      isPending={isPending}
      isError={isError}
    />
  );
};
