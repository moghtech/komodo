import { lazy, ReactNode, Suspense } from "react";
import { usePermissions, useRead } from "@/lib/hooks";
import { Types } from "komodo_client";
import { Section } from "mogh_ui";
import ServerProcesses from "./processes";
import ServerContainerStats from "./containers";
import ServerDisks from "./disks";
import ServerCurrentStats from "./current";
import ServerSystemInfo from "./system-info";
import { useIsServerAvailable } from "../hooks";

// Loaded lazily to keep recharts out of the entry chunk.
const ServerHistoricalStats = lazy(() => import("./historical"));

export default function ServerStats({
  id,
  titleOther,
}: {
  id: string;
  titleOther?: ReactNode;
}) {
  const { specific } = usePermissions({ type: "Server", id });
  const isServerAvailable = useIsServerAvailable(id);

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

      <Suspense fallback={null}>
        <ServerHistoricalStats id={id} />
      </Suspense>

      <ServerContainerStats id={id} />

      <ServerDisks stats={stats} />

      {specific.includes(Types.SpecificPermission.Processes) && (
        <ServerProcesses id={id} />
      )}
    </Section>
  );
}
