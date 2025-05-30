import { usePermissions, useRead } from "@lib/hooks";
import { ReactNode } from "react";
import { Types } from "komodo_client";
import { Section } from "@components/layouts";
import { InspectContainerView } from "@components/inspect";

export const ContainerInspect = ({
  id,
  container,
  titleOther,
}: {
  id: string;
  container: string;
  titleOther: ReactNode;
}) => {
  const { specific } = usePermissions({ type: "Server", id });
  if (!specific.includes(Types.SpecificPermission.Inspect)) {
    return (
      <Section titleOther={titleOther}>
        <div className="min-h-[60vh]">
          <h1>User does not have permission to inspect this Server.</h1>
        </div>
      </Section>
    );
  }
  return (
    <Section titleOther={titleOther}>
      <ContainerInspectInner id={id} container={container} />
    </Section>
  );
};

const ContainerInspectInner = ({
  id,
  container,
}: {
  id: string;
  container: string;
}) => {
  const {
    data: inspect_container,
    error,
    isPending,
    isError,
  } = useRead("InspectDockerContainer", {
    server: id,
    container,
  });
  return (
    <InspectContainerView
      container={inspect_container}
      error={error}
      isPending={isPending}
      isError={isError}
    />
  );
};
