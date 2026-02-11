import { DataTable } from "@/ui/data-table";
import Section from "@/ui/section";
import { useServer } from "..";
import { Types } from "komodo_client";
import { useRead } from "@/lib/hooks";

export default function ServerSystemInfo({
  id,
  stats,
}: {
  id: string;
  stats: Types.SystemStats | undefined;
}) {
  const isServerAvailable = useServer(id)?.info.state === Types.ServerState.Ok;
  const info = useRead(
    "GetSystemInformation",
    { server: id },
    { enabled: isServerAvailable },
  ).data;
  const diskTotal = stats?.disks.reduce(
    (acc, curr) => (acc += curr.total_gb),
    0,
  );
  return (
    <Section title="System Info">
      <DataTable
        tableKey="system-info"
        data={
          info
            ? [
                {
                  ...info,
                  memTotal: stats?.mem_total_gb,
                  diskTotal,
                },
              ]
            : []
        }
        columns={[
          {
            header: "Hostname",
            accessorKey: "host_name",
          },
          {
            header: "Os",
            accessorKey: "os",
          },
          {
            header: "Kernel",
            accessorKey: "kernel",
          },
          {
            header: "CPU",
            accessorKey: "cpu_brand",
          },
          {
            header: "Core Count",
            accessorFn: ({ core_count }) =>
              `${core_count} Core${(core_count || 0) > 1 ? "s" : ""}`,
          },
          {
            header: "Total Memory",
            accessorFn: ({ memTotal }) => `${memTotal?.toFixed(2)} GB`,
          },
          {
            header: "Total Disk Size",
            accessorFn: ({ diskTotal }) => `${diskTotal?.toFixed(2)} GB`,
          },
        ]}
      />
    </Section>
  );
}
