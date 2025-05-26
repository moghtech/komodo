import { Section } from "@components/layouts";
import { ResourceLink } from "@components/resources/common";
import { useServer } from "@components/resources/server";
import {
  ConfirmButton,
  DOCKER_LINK_ICONS,
  DockerContainersSection,
  DockerLabelsSection,
  DockerOptions,
  DockerResourcePageName,
  ShowHideButton,
} from "@components/util";
import { useExecute, useRead, useSetTitle } from "@lib/hooks";
import { has_minimum_permissions } from "@lib/utils";
import { Types } from "komodo_client";
import { Badge } from "@ui/badge";
import { Button } from "@ui/button";
import { DataTable } from "@ui/data-table";
import { ChevronLeft, Info, Loader2, SearchCode, Trash } from "lucide-react";
import { useNavigate, useParams } from "react-router-dom";
import { useState } from "react";
import { MonacoEditor } from "@components/monaco";

export const VolumePage = () => {
  const { type, id, volume } = useParams() as {
    type: string;
    id: string;
    volume: string;
  };
  if (type !== "servers") {
    return <div>This resource type does not have any volumes.</div>;
  }
  return <VolumePageInner id={id} volume={decodeURIComponent(volume)} />;
};

const VolumePageInner = ({
  id,
  volume: volume_name,
}: {
  id: string;
  volume: string;
}) => {
  const [showInspect, setShowInspect] = useState(false);
  const server = useServer(id);
  useSetTitle(`${server?.name} | volume | ${volume_name}`);
  const nav = useNavigate();

  const perms = useRead("GetPermission", {
    target: { type: "Server", id },
  }).data;

  const {
    data: volume,
    isPending,
    isError,
  } = useRead("InspectDockerVolume", {
    server: id,
    volume: volume_name,
  });

  const containers = useRead(
    "ListDockerContainers",
    {
      server: id,
    },
    { refetchInterval: 10_000 }
  ).data?.filter((container) => container.volumes?.includes(volume_name));

  const { mutate: deleteVolume, isPending: deletePending } = useExecute(
    "DeleteVolume",
    {
      onSuccess: () => nav("/servers/" + id),
    }
  );

  if (isPending) {
    return (
      <div className="flex justify-center w-full py-4">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  if (isError) {
    return <div className="flex w-full py-4">Failed to inspect volume.</div>;
  }

  if (!volume) {
    return (
      <div className="flex w-full py-4">
        No volume found with given name: {volume_name}
      </div>
    );
  }

  const canExecute = has_minimum_permissions(
    perms,
    Types.PermissionLevel.Execute
  );

  const unused = containers && containers.length === 0 ? true : false;

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
        </div>

        {/* TITLE */}
        <div className="flex items-center gap-4">
          <div className="mt-1">
            <DOCKER_LINK_ICONS.volume
              server_id={id}
              name={volume_name}
              size={8}
            />
          </div>
          <DockerResourcePageName name={volume_name} />
          {containers && containers.length === 0 && (
            <Badge variant="destructive">Unused</Badge>
          )}
        </div>

        {/* INFO */}
        <div className="flex flex-wrap gap-4 items-center text-muted-foreground">
          <ResourceLink type="Server" id={id} />
        </div>
      </div>

      {/* MAYBE DELETE */}
      {canExecute && unused && (
        <ConfirmButton
          variant="destructive"
          title="Delete Volume"
          icon={<Trash className="w-4 h-4" />}
          loading={deletePending}
          onClick={() => deleteVolume({ server: id, name: volume_name })}
        />
      )}

      {containers && containers.length > 0 && (
        <DockerContainersSection server_id={id} containers={containers} />
      )}

      {/* TOP LEVEL VOLUME INFO */}
      <Section title="Details" icon={<Info className="w-4 h-4" />}>
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
        <DockerOptions options={volume.Options} />
      </Section>

      <DockerLabelsSection labels={volume.Labels} />

      {perms?.specific.includes(Types.SpecificPermission.DockerInspect) && (
        <Section
          title="Inspect"
          icon={<SearchCode className="w-4 h-4" />}
          titleRight={
            <div className="pl-2">
              <ShowHideButton show={showInspect} setShow={setShowInspect} />
            </div>
          }
        >
          {showInspect && (
            <MonacoEditor
              value={JSON.stringify(volume, null, 2)}
              language="json"
              readOnly
            />
          )}
        </Section>
      )}
    </div>
  );
};
