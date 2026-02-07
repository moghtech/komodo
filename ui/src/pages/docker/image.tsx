import ContainersSection from "@/components/docker/containers-section";
import DockerLabelsSection from "@/components/docker/labels-section";
import { MonacoEditor } from "@/components/monaco";
import ResourceUpdates from "@/components/updates/resource";
import { hexColorByIntention } from "@/lib/color";
import { fmtDateWithMinutes, fmtSizeBytes } from "@/lib/formatting";
import { useExecute, usePermissions, useRead, useSetTitle } from "@/lib/hooks";
import { ResourceDescription, ResourceLink } from "@/resources/common";
import { ServerComponents } from "@/resources/server";
import { ICONS } from "@/theme/icons";
import ConfirmButton from "@/ui/confirm-button";
import { DataTable } from "@/ui/data-table";
import DividedChildren from "@/ui/divided-children";
import EntityHeader from "@/ui/entity-header";
import EntityPage from "@/ui/entity-page";
import Section from "@/ui/section";
import ShowHideButton from "@/ui/show-hide-button";
import { Box, Center, Group, Loader, Stack, Text } from "@mantine/core";
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
  const [showInspect, setShowInspect] = useState(false);
  const [showHistory, setShowHistory] = useState(false);
  const server = ServerComponents.useListItem(serverId);
  useSetTitle(`${server?.name} | Image | ${imageName}`);
  const nav = useNavigate();

  const { canExecute, specific } = usePermissions({
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

  if (isPending) {
    return (
      <Center h="30vh">
        <Loader size="xl" />
      </Center>
    );
  }

  if (isError) {
    return (
      <Center h="30vh">
        <Text>Failed to inspect image.</Text>
      </Center>
    );
  }

  if (!image) {
    return (
      <Center h="30vh">
        <Text>No image found with given name: {imageName}</Text>
      </Center>
    );
  }

  const unused = containers && containers.length === 0 ? true : false;
  const intention = unused ? "Critical" : "Good";

  const Header = (
    <Stack justify="space-between">
      <Stack
        gap="md"
        pb="md"
        bd="1px solid var(--mantine-color-accent-border-0)"
        bdrs="md"
      >
        <EntityHeader
          name={imageName}
          icon={
            <ICONS.Image size="2rem" color={hexColorByIntention(intention)} />
          }
          intent={intention}
          state={unused ? "Unused" : "In Use"}
        />
        <DividedChildren px="md">
          <ResourceLink type="Server" id={serverId} />
          {image.Id && (
            <Group gap="xs">
              <Text>Id:</Text>
              <Text title={image.Id} maw={150} className="text-ellipsis">
                {image.Id}
              </Text>
            </Group>
          )}
        </DividedChildren>
      </Stack>
      <ResourceDescription type="Server" id={serverId} />
    </Stack>
  );

  return (
    <EntityPage backTo={"/servers/" + serverId}>
      <Stack hiddenFrom="xl" w="100%">
        {Header}
        <ResourceUpdates type="Server" id={serverId} />
      </Stack>
      <Group
        visibleFrom="xl"
        gap="xl"
        w="100%"
        align="stretch"
        grow
        preventGrowOverflow={false}
      >
        {Header}
        <ResourceUpdates type="Server" id={serverId} />
      </Group>

      <Stack mt="lg" gap="xl">
        {canExecute && unused && (
          <Section
            title="Execute"
            icon={<ICONS.Execution size="1rem" />}
            my="xl"
          >
            <ConfirmButton
              color="red"
              icon={<ICONS.Delete size="1rem" />}
              loading={deletePending}
              onClick={() => deleteImage({ server: serverId, name: imageName })}
            >
              Delete Image
            </ConfirmButton>
          </Section>
        )}

        {containers && containers.length > 0 && (
          <ContainersSection serverId={serverId} containers={containers} />
        )}

        {/* TOP LEVEL IMAGE INFO */}
        <Section title="Details" icon={<ICONS.Info size="1rem" />}>
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
            icon={<ICONS.History size="1rem" />}
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
                      fmtDateWithMinutes(new Date(row.original.Created * 1000)),
                    size: 200,
                  },
                ]}
              />
            )}
          </Section>
        )}

        {specific.includes(Types.SpecificPermission.Inspect) && (
          <Section
            title="Inspect"
            icon={<ICONS.Search size="1rem" />}
            titleRight={
              <Box pl="md">
                <ShowHideButton show={showInspect} setShow={setShowInspect} />
              </Box>
            }
          >
            {showInspect && (
              <MonacoEditor
                value={JSON.stringify(image, null, 2)}
                language="json"
                readOnly
              />
            )}
          </Section>
        )}

        <DockerLabelsSection labels={image?.Config?.Labels} />
      </Stack>
    </EntityPage>
  );
}
