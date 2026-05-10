import { useRead, useResourceName, useSelectedResources } from "@/lib/hooks";
import { Types } from "komodo_client";
import { ICONS } from "@/lib/icons";
import { Group, BoxProps } from "@mantine/core";
import TableTags from "@/components/tags/table";
import { DataTable, SortableHeader } from "mogh_ui";
import { DeploymentComponents } from ".";
import ResourceLink from "@/resources/link";
import DeploymentUpdateAvailable from "./update-available";

export default function DeploymentTable({
  resources,
  ...boxProps
}: {
  resources: Types.DeploymentListItem[];
} & BoxProps) {
  const swarmName = useResourceName("Swarm");
  const serverName = useResourceName("Server");

  const [_, setSelectedResources] = useSelectedResources("Deployment");

  return (
    <DataTable
      {...boxProps}
      tableKey="deployments"
      data={resources}
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
            <Group wrap="nowrap">
              <ResourceLink type="Deployment" id={row.original.id} />
              <DeploymentUpdateAvailable id={row.original.id} small />
            </Group>
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
          }) => <Image buildId={build_id} image={image} />,
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
  const builds = useRead("ListBuilds", {}).data;
  if (buildId) {
    const build = builds?.find((build) => build.id === buildId);
    if (build) {
      return <ResourceLink type="Build" id={buildId} />;
    } else {
      return undefined;
    }
  } else {
    const [img] = image.split(":");
    return (
      <Group wrap="nowrap">
        <ICONS.Image size="1rem" />
        {img}
      </Group>
    );
  }
};
