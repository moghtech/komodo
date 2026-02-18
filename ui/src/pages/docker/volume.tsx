import ContainersSection from "@/components/docker/containers-section";
import DockerLabelsSection from "@/components/docker/labels-section";
import DockerOptions from "@/components/docker/options";
import InspectSection from "@/components/inspect-section";
import { useExecute, usePermissions, useRead, useSetTitle } from "@/lib/hooks";
import { useServer } from "@/resources/server";
import ResourceSubPage from "@/resources/sub-page";
import { ICONS } from "@/theme/icons";
import ConfirmButton from "@/ui/confirm-button";
import { DataTable } from "@/ui/data-table";
import Section from "@/ui/section";
import { Center, Group, Loader, Text } from "@mantine/core";
import { Types } from "komodo_client";
import { useNavigate, useParams } from "react-router-dom";

export default function Volume() {
  const { type, id, volume } = useParams() as {
    type: string;
    id: string;
    volume: string;
  };
  if (type !== "servers") {
    return (
      <Center h="50vh">
        <Text>This resource type does not have any volumes.</Text>
      </Center>
    );
  }
  return <VolumeInner serverId={id} volumeName={volume} />;
}

function VolumeInner({
  serverId,
  volumeName,
}: {
  serverId: string;
  volumeName: string;
}) {
  const server = useServer(serverId);
  useSetTitle(`${server?.name} | Volume | ${volumeName}`);
  const nav = useNavigate();

  const { specific } = usePermissions({
    type: "Server",
    id: serverId,
  });

  const {
    data: volume,
    isPending,
    isError,
  } = useRead("InspectDockerVolume", {
    server: serverId,
    volume: volumeName,
  });

  const { mutate: deleteVolume, isPending: deletePending } = useExecute(
    "DeleteVolume",
    {
      onSuccess: () => nav("/servers/" + serverId),
    },
  );

  const containers = useRead(
    "ListDockerContainers",
    {
      server: serverId,
    },
    { refetchInterval: 10_000 },
  ).data?.filter((container) => container.volumes?.includes(volumeName));

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
        <Text>Failed to inspect volume.</Text>
      </Center>
    );
  }

  if (!volume) {
    return (
      <Center h="30vh">
        <Text>No volume found with given name: {volumeName}</Text>
      </Center>
    );
  }

  const unused = containers && containers.length === 0 ? true : false;

  const intention = unused ? "Critical" : "Good";

  return (
    <ResourceSubPage
      entityTypeName="Volume"
      parentType="Server"
      parentId={serverId}
      name={volumeName}
      icon={ICONS.Volume}
      intent={intention}
      state={unused ? "Unused" : "In Use"}
      info={
        <>
          {volume.Scope && (
            <Group gap="xs">
              <Text c="dimmed">Scope:</Text>
              <Text>{volume.Scope}</Text>
            </Group>
          )}
        </>
      }
      executions={
        <>
          {unused && (
            <ConfirmButton
              color="red"
              icon={<ICONS.Delete size="1rem" />}
              loading={deletePending}
              onClick={() =>
                deleteVolume({ server: serverId, name: volumeName })
              }
            >
              Delete Volume
            </ConfirmButton>
          )}
        </>
      }
    >
      {containers && containers.length > 0 && (
        <ContainersSection serverId={serverId} containers={containers} />
      )}

      {/* TOP LEVEL NETWORK INFO */}
      <Section title="Details" icon={<ICONS.Info size="1.3rem" />}>
        <DataTable
          tableKey="volume-info"
          data={[volume]}
          columns={[
            {
              accessorKey: "Driver",
              header: "Driver",
            },
            {
              accessorKey: "Scope",
              header: "Scope",
            },
            {
              accessorKey: "CreatedAt",
              header: "Created At",
            },
            {
              accessorKey: "UsageData.Size",
              header: "Used Size",
            },
          ]}
        />
        {volume.Options && (
          <DockerOptions options={Object.entries(volume.Options)} />
        )}
      </Section>

      {specific.includes(Types.SpecificPermission.Inspect) && (
        <InspectSection json={volume} showToggle />
      )}

      <DockerLabelsSection labels={volume?.Labels} />
    </ResourceSubPage>
  );
}
