import { ReactNode } from "react";
import { usePermissions, useRead } from "@/lib/hooks";
import { useServer } from "..";
import { Types } from "komodo_client";
import Section from "@/ui/section";
import ServerProcesses from "./processes";
import ServerContainerStats from "./containers";
import ServerDisks from "./disks";
import ServerCurrentStats from "./current";
import ServerHistoricalStats from "./historical";
import ServerSystemInfo from "./system-info";

export default function ServerStats({
  id,
  titleOther,
}: {
  id: string;
  titleOther?: ReactNode;
}) {
  const { specific } = usePermissions({ type: "Server", id });
  const isServerAvailable = useServer(id)?.info.state === Types.ServerState.Ok;

  const stats = useRead(
    "GetSystemStats",
    { server: id },
    {
      enabled: isServerAvailable,
      refetchInterval: 10_000,
    },
  ).data;

  return (
    <Section titleOther={titleOther} gap="2.5rem">
      <ServerSystemInfo id={id} stats={stats} />

      <ServerCurrentStats id={id} stats={stats} />

      <ServerContainerStats id={id} />

      <ServerDisks stats={stats} />

      {specific.includes(Types.SpecificPermission.Processes) && (
        <ServerProcesses id={id} />
      )}

      <ServerHistoricalStats id={id} />
    </Section>
  );
}
