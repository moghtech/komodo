import { TableTags } from "@components/tags";
import { Types } from "komodo_client";
import { DataTable, SortableHeader } from "@ui/data-table";
import { useRead, useResourceName, useSelectedResources } from "@lib/hooks";
import { ResourceLink } from "../common";
import { DeploymentComponents, UpdateAvailable } from ".";
import { HardDrive } from "lucide-react";

export const DeploymentTable = ({
  deployments,
}: {
  deployments: Types.DeploymentListItem[];
}) => {
  const swarmName = useResourceName("Swarm");
  const serverName = useResourceName("Server");

  const [_, setSelectedResources] = useSelectedResources("Deployment");

  return (
    <DataTable
      tableKey="deployments"
      data={deployments}
      selectOptions={{
        selectKey: ({ name }) => name,
        onSelect: setSelectedResources,
      }}
      columns={[
        {
          accessorKey: "name",
          header: ({ column }) => (
            <SortableHeader column={column} title="Name" />
          ),
          cell: ({ row }) => (
            <div className="flex items-center justify-between gap-2">
              <ResourceLink type="Deployment" id={row.original.id} />
              <UpdateAvailable id={row.original.id} small />
            </div>
          ),
          size: 200,
        },
        {
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
          }) => <Image build_id={build_id} image={image} />,
          size: 200,
        },
        {
          header: ({ column }) => (
            <SortableHeader column={column} title="Host" />
          ),
          accessorKey: "info.server_id",
          sortingFn: (a, b) => {
            const name_a = a.original.info.swarm_id
              ? swarmName(a.original.info.swarm_id)
              : serverName(a.original.info.server_id);
            const name_b = b.original.info.swarm_id
              ? swarmName(b.original.info.swarm_id)
              : serverName(b.original.info.server_id);

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
          cell: ({ row }) => <TableTags tag_ids={row.original.tags} />,
        },
      ]}
    />
  );
};

const Image = ({
  build_id,
  image,
}: {
  build_id: string | undefined;
  image: string;
}) => {
  const builds = useRead("ListBuilds", {}).data;
  if (build_id) {
    const build = builds?.find((build) => build.id === build_id);
    if (build) {
      return <ResourceLink type="Build" id={build_id} />;
    } else {
      return undefined;
    }
  } else {
    const [img] = image.split(":");
    return (
      <div className="flex gap-2 items-center">
        <HardDrive className="w-4 h-4" />
        {img}
      </div>
    );
  }
};
