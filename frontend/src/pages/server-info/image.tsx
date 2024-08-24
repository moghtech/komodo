import { Section } from "@components/layouts";
import { ResourceLink } from "@components/resources/common";
import { useServer } from "@components/resources/server";
import {
  DOCKER_LINK_ICONS,
  DockerContainersSection,
  DockerLabelsSection,
  DockerResourcePageName,
} from "@components/util";
import { fmt_date_with_minutes, format_size_bytes } from "@lib/formatting";
import { useRead, useSetTitle } from "@lib/hooks";
import { Badge } from "@ui/badge";
import { Button } from "@ui/button";
import { DataTable } from "@ui/data-table";
import { ChevronLeft, HistoryIcon, Info, Loader2 } from "lucide-react";
import { useNavigate, useParams } from "react-router-dom";

export const ImagePage = () => {
  const { type, id, image } = useParams() as {
    type: string;
    id: string;
    image: string;
  };
  if (type !== "servers") {
    return <div>This resource type does not have any images.</div>;
  }
  return <ImagePageInner id={id} image={decodeURIComponent(image)} />;
};

const ImagePageInner = ({
  id,
  image: image_name,
}: {
  id: string;
  image: string;
}) => {
  const server = useServer(id);
  useSetTitle(`${server?.name} | image | ${image_name}`);
  const nav = useNavigate();
  // const perms = useRead("GetPermissionLevel", {
  //   target: { type: "Server", id },
  // }).data;
  const {
    data: image,
    isPending,
    isError,
  } = useRead("InspectDockerImage", {
    server: id,
    image: image_name,
  });
  const containers = useRead(
    "ListDockerContainers",
    {
      server: id,
    },
    { refetchInterval: 5000 }
  ).data?.filter((container) =>
    !image?.Id ? false : container.image_id === image?.Id
  );
  const history = useRead("ListDockerImageHistory", {
    server: id,
    image: image_name,
  }).data;

  if (isPending) {
    return (
      <div className="flex justify-center w-full py-4">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }
  if (isError) {
    return <div className="flex w-full py-4">Failed to inspect image.</div>;
  }
  if (!image) {
    return (
      <div className="flex w-full py-4">
        No image found with given name: {image_name}
      </div>
    );
  }

  // const disabled = !has_minimum_permissions(perms, Types.PermissionLevel.Write);

  return (
    <div className="flex flex-col gap-16 mb-24">
      {/* HEADER */}
      <div className="flex flex-col gap-4">
        {/* BACK */}
        <div className="flex items-center justify-between mb-4">
          <Button
            className="gap-2"
            variant="secondary"
            onClick={() => nav("/servers/" + id)}
          >
            <ChevronLeft className="w-4" /> Back
          </Button>

          {/* <Button className="gap-2" variant="destructive">
            <Trash className="w-4" /> Delete
          </Button> */}
        </div>

        {/* TITLE */}
        <div className="flex items-center gap-4">
          <div className="mt-1">
            <DOCKER_LINK_ICONS.image server_id={id} name={image.Id} size={8} />
          </div>
          <DockerResourcePageName name={image_name} />
          {containers && containers.length === 0 && (
            <Badge variant="destructive">Unused</Badge>
          )}
        </div>

        {/* INFO */}
        <div className="flex flex-wrap gap-4 items-center text-muted-foreground">
          <ResourceLink type="Server" id={id} />
          {image.Id ? (
            <>
              |
              <div className="flex gap-2">
                Id:
                <div
                  title={image.Id}
                  className="max-w-[150px] overflow-hidden text-ellipsis"
                >
                  {image.Id}
                </div>
              </div>
            </>
          ) : null}
        </div>
      </div>

      {/* TOP LEVEL IMAGE INFO */}
      <Section title="Details" icon={<Info className="w-4 h-4" />}>
        <DataTable
          tableKey="image-info"
          data={[image]}
          columns={[
            {
              accessorKey: "Architecture",
              header: "Architecture",
            },
            {
              accessorKey: "Os",
              header: "Os",
            },
            {
              accessorKey: "Size",
              header: "Size",
              cell: ({ row }) =>
                row.original.Size
                  ? format_size_bytes(row.original.Size)
                  : "Unknown",
            },
          ]}
        />
      </Section>

      {containers && containers.length > 0 && (
        <DockerContainersSection server_id={id} containers={containers} />
      )}

      {history && history.length > 0 && (
        <Section title="History" icon={<HistoryIcon className="w-4 h-4" />}>
          <DataTable
            tableKey="image-history"
            data={history.toReversed()}
            columns={[
              {
                accessorKey: "CreatedBy",
                header: "Created By",
                size: 400,
              },
              {
                accessorKey: "Created",
                header: "Timestamp",
                cell: ({ row }) =>
                  fmt_date_with_minutes(new Date(row.original.Created * 1000)),
                size: 200,
              },
            ]}
          />
        </Section>
      )}

      <DockerLabelsSection labels={image?.Config?.Labels} />
    </div>
  );
};
