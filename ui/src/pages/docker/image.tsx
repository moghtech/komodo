import ContainersSection from "@/components/docker/containers-section";
import DockerLabelsSection from "@/components/docker/labels-section";
import InspectSection from "@/components/inspect-section";
import { fmtDateWithMinutes, fmtSizeBytes } from "@/lib/formatting";
import { useExecute, usePermissions, useRead, useSetTitle } from "@/lib/hooks";
import { useServer } from "@/resources/server";
import ResourceSubPage from "@/resources/sub-page";
import { ICONS } from "@/lib/icons";
import ConfirmButton from "@/ui/confirm-button";
import { DataTable } from "@/ui/data-table";
import PageGuard from "@/ui/page-guard";
import Section from "@/ui/section";
import ShowHideButton from "@/ui/show-hide-button";
import { Box, Center, Group, Text } from "@mantine/core";
import { Types } from "komodo_client";
import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";

export default function Image() {
  const { type, id, image } = useParams() as {
    type: string;
    id: string;
    image: string;
  };
  if (type !== "servers") {
    return (
      <Center h="50vh">
        <Text>This resource type does not have any images.</Text>
      </Center>
    );
  }
  return <ImageInner serverId={id} imageName={image} />;
}

function ImageInner({
  serverId,
  imageName,
}: {
  serverId: string;
  imageName: string;
}) {
  const [showHistory, setShowHistory] = useState(false);
  const server = useServer(serverId);
  useSetTitle(`${server?.name} | Image | ${imageName}`);
  const nav = useNavigate();

  const { specific } = usePermissions({
    type: "Server",
    id: serverId,
  });

  const {
    data: image,
    isPending,
    isError,
  } = useRead("InspectDockerImage", {
    server: serverId,
    image: imageName,
  });

  const containers = useRead(
    "ListDockerContainers",
    {
      server: serverId,
    },
    { refetchInterval: 10_000 },
  ).data?.filter((container) =>
    !image?.Id ? false : container.image_id === image?.Id,
  );

  const history = useRead("ListDockerImageHistory", {
    server: serverId,
    image: imageName,
  }).data;

  const { mutate: deleteImage, isPending: deletePending } = useExecute(
    "DeleteImage",
    {
      onSuccess: () => nav("/servers/" + serverId),
    },
  );

  const unused = containers && containers.length === 0 ? true : false;
  const intention = unused ? "Critical" : "Good";

  return (
    <PageGuard
      isPending={isPending}
      error={
        isError
          ? "Failed to inspect image."
          : !image
            ? "No image found with name: " + imageName
            : undefined
      }
    >
      {image && (
        <ResourceSubPage
          entityTypeName="Image"
          parentType="Server"
          parentId={serverId}
          name={imageName}
          icon={ICONS.Image}
          intent={intention}
          state={unused ? "Unused" : "In Use"}
          info={
            image.Id && (
              <Group gap="xs">
                <Text>Id:</Text>
                <Text title={image.Id} maw={150} className="text-ellipsis">
                  {image.Id}
                </Text>
              </Group>
            )
          }
          executions={
            unused && (
              <ConfirmButton
                variant="filled"
                color="red"
                icon={<ICONS.Delete size="1rem" />}
                loading={deletePending}
                onClick={() =>
                  deleteImage({ server: serverId, name: imageName })
                }
              >
                Delete Image
              </ConfirmButton>
            )
          }
        >
          {containers && containers.length > 0 && (
            <ContainersSection serverId={serverId} containers={containers} />
          )}

          {/* TOP LEVEL IMAGE INFO */}
          <Section title="Details" icon={<ICONS.Info size="1.3rem" />}>
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
                      ? fmtSizeBytes(row.original.Size)
                      : "Unknown",
                },
              ]}
            />
          </Section>

          {history && history.length > 0 && (
            <Section
              title="History"
              icon={<ICONS.History size="1.3rem" />}
              titleRight={
                <Box pl="md">
                  <ShowHideButton show={showHistory} setShow={setShowHistory} />
                </Box>
              }
            >
              {showHistory && (
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
                        fmtDateWithMinutes(
                          new Date(row.original.Created * 1000),
                        ),
                      size: 200,
                    },
                  ]}
                />
              )}
            </Section>
          )}

          {specific.includes(Types.SpecificPermission.Inspect) && (
            <InspectSection json={image} showToggle />
          )}

          <DockerLabelsSection labels={image?.Config?.Labels} />
        </ResourceSubPage>
      )}
    </PageGuard>
  );
}
