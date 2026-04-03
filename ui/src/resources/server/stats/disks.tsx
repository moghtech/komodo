import { ICONS } from "@/lib/icons";
import { DataTable, SortableHeader } from "@/ui/data-table";
import { Section } from "mogh_ui";
import ShowHideButton from "@/ui/show-hide-button";
import { Group, Text } from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";

export default function ServerDisks({
  stats,
}: {
  stats: Types.SystemStats | undefined;
}) {
  const [show, setShow] = useLocalStorage({
    key: "server-stats-disks-show-v2",
    defaultValue: true,
  });
  const diskUsed = stats?.disks.reduce((acc, curr) => (acc += curr.used_gb), 0);
  const diskTotal = stats?.disks.reduce(
    (acc, curr) => (acc += curr.total_gb),
    0,
  );

  return (
    <Section
      withBorder
      title="Disks"
      icon={<ICONS.Disk size="1.3rem" />}
      titleRight={
        <Group ml={{ md: "xl" }} gap="md">
          <Group gap="xs" wrap="nowrap">
            <Text c="dimmed">Used:</Text>
            <Text>{diskUsed?.toFixed(2)} GB</Text>
          </Group>
          <Group gap="xs" wrap="nowrap">
            <Text c="dimmed">Total:</Text>
            <Text>{diskTotal?.toFixed(2)} GB</Text>
          </Group>
          <ShowHideButton show={show} setShow={setShow} />
        </Group>
      }
      onHeaderClick={() => setShow((s) => !s)}
    >
      {show && (
        <DataTable
          sortDescFirst
          tableKey="server-disks"
          data={
            stats?.disks.map((disk) => ({
              ...disk,
              percentage: 100 * (disk.used_gb / disk.total_gb),
            })) ?? []
          }
          columns={[
            {
              header: "Path",
              cell: ({ row }) => (
                <div className="overflow-hidden overflow-ellipsis">
                  {row.original.mount}
                </div>
              ),
            },
            {
              accessorKey: "used_gb",
              header: ({ column }) => (
                <SortableHeader column={column} title="Used" sortDescFirst />
              ),
              cell: ({ row }) => <>{row.original.used_gb.toFixed(2)} GB</>,
            },
            {
              accessorKey: "total_gb",
              header: ({ column }) => (
                <SortableHeader column={column} title="Total" sortDescFirst />
              ),
              cell: ({ row }) => <>{row.original.total_gb.toFixed(2)} GB</>,
            },
            {
              accessorKey: "percentage",
              header: ({ column }) => (
                <SortableHeader
                  column={column}
                  title="Percentage"
                  sortDescFirst
                />
              ),
              cell: ({ row }) => (
                <>{row.original.percentage.toFixed(2)}% Full</>
              ),
            },
          ]}
        />
      )}
    </Section>
  );
}
