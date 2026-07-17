import { useResourceSelectionState } from "@/lib/hooks";
import { Types } from "komodo_client";
import { ICONS } from "@/lib/icons";
import { Group, BoxProps } from "@mantine/core";
import TableTags from "@/components/tags/table";
import { DataTable, SortableHeader } from "mogh_ui";
import { DeploymentComponents } from ".";
import ResourceLink from "@/resources/link";
import DeploymentUpdateAvailable from "./update-available";

const SORT_KEYS = ["Name", "Image", "Host", "State"];

export default function DeploymentTable({
  resources,
  onServerSort,
  ...boxProps
}: {
  resources: Types.DeploymentListItem[];
  /** When provided, sorting is handled server side,
   * and sort updates are passed to this callback. */
  onServerSort?: (sort: {
    sort_by?: string;
    sort_desc?: boolean;
  }) => void;
} & BoxProps) {
  const selectionState = useResourceSelectionState("Deployment");

  return (
    <DataTable
      {...boxProps}
      manualSorting={!!onServerSort}
      onSortingStateChange={
        onServerSort &&
        ((sorting) => {
          const sort = sorting.find((s) => SORT_KEYS.includes(s.id));
          onServerSort(
            sort ? { sort_by: sort.id, sort_desc: sort.desc } : {},
          );
        })
      }
      tableKey="deployments"
      data={resources}
      selectOptions={{
        selectKey: ({ name }) => name,
        state: selectionState,
      }}
      columns={[
        {
          id: "Name",
          accessorKey: "name",
          header: ({ column }) => (
            <SortableHeader column={column} title="Name" />
          ),
          cell: ({ row }) => (
            <Group wrap="nowrap">
              <ResourceLink type="Deployment" id={row.original.id} />
              <DeploymentUpdateAvailable id={row.original.id} small />
            </Group>
          ),
          size: 200,
        },
        {
          id: "Image",
          accessorKey: "info.image",
          header: ({ column }) => (
            <SortableHeader column={column} title="Image" />
          ),
          cell: ({
            row: {
              original: {
                info: { build_id, image },
              },
            },
          }) => <Image buildId={build_id} image={image} />,
          size: 200,
        },
        {
          header: ({ column }) => (
            <SortableHeader column={column} title="Host" />
          ),
          id: "Host",
          accessorKey: "info.server_id",
          sortingFn: (a, b) => {
            const name_a = a.original.info.swarm_id
              ? a.original.info.swarm_name
              : a.original.info.server_name;
            const name_b = b.original.info.swarm_id
              ? b.original.info.swarm_name
              : b.original.info.server_name;

            if (!name_a && !name_b) return 0;
            if (!name_a) return 1;
            if (!name_b) return -1;

            if (name_a > name_b) return 1;
            else if (name_a < name_b) return -1;
            else return 0;
          },
          cell: ({ row }) =>
            row.original.info.swarm_id ? (
              <ResourceLink type="Swarm" id={row.original.info.swarm_id} />
            ) : (
              <ResourceLink type="Server" id={row.original.info.server_id} />
            ),
          size: 200,
        },
        {
          id: "State",
          accessorKey: "info.state",
          header: ({ column }) => (
            <SortableHeader column={column} title="State" />
          ),
          cell: ({ row }) => (
            <DeploymentComponents.State id={row.original.id} />
          ),
          size: 120,
        },
        {
          header: "Tags",
          cell: ({ row }) => <TableTags tagIds={row.original.tags} />,
        },
      ]}
    />
  );
}

const Image = ({
  buildId,
  image,
}: {
  buildId: string | undefined;
  image: string;
}) => {
  if (buildId) {
    return <ResourceLink type="Build" id={buildId} />;
  } else {
    const img = image?.split(":")?.[0];
    if (img) {
      return (
        <Group wrap="nowrap">
          <ICONS.Image size="1rem" />
          {img}
        </Group>
      );
    }
  }
};
