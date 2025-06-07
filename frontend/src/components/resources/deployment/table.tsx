import { TableTags } from "@components/tags";
import { Types } from "komodo_client";
import { DataTable, SortableHeader } from "@ui/data-table";
import { useRead, useSelectedResources } from "@lib/hooks";
import { ResourceLink } from "../common";
import { DeploymentComponents, UpdateAvailable } from ".";
import { HardDrive } from "lucide-react";
import { useCallback } from "react";

export const DeploymentTable = ({
  deployments,
}: {
  deployments: Types.DeploymentListItem[];
}) => {
  const servers = useRead("ListServers", {}).data;
  const serverName = useCallback(
    (id: string) => servers?.find((server) => server.id === id)?.name,
    [servers]
  );

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
          accessorKey: "info.server_id",
          sortingFn: (a, b) => {
            const sa = serverName(a.original.info.server_id);
            const sb = serverName(b.original.info.server_id);

            if (!sa && !sb) return 0;
            if (!sa) return 1;
            if (!sb) return -1;

            if (sa > sb) return 1;
            else if (sa < sb) return -1;
            else return 0;
          },
          header: ({ column }) => (
            <SortableHeader column={column} title="Server" />
          ),
          cell: ({ row }) => (
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
