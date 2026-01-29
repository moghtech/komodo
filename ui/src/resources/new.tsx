import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useRead, useWrite } from "@/lib/hooks";
import { UsableResource } from ".";
import { Types } from "komodo_client";
import { notifications } from "@mantine/notifications";
import { usableResourcePath } from "@/lib/utils";
import CreateModal from "@/ui/create-modal";
import { TextInput } from "@mantine/core";

export interface NewResourceProps {
  type: UsableResource;
  readableType?: string;
  swarmId?: string;
  serverId?: string;
  builderId?: string;
  buildId?: string;
  name?: string;
  selectSwarm?: boolean;
  selectServer?: boolean;
  selectBuilder?: boolean;
}

export default function NewResource({
  type,
  readableType,
  swarmId: _swarmId,
  serverId: _serverId,
  builderId: _builderId,
  buildId,
  name: _name = "",
  selectSwarm,
  selectServer,
  selectBuilder,
}: NewResourceProps) {
  const nav = useNavigate();
  const showTemplateSelector =
    (useRead(`List${type}s`, {}).data?.filter((r) => r.template).length ?? 0) >
    0;
  const swarmsExist = useRead("ListSwarms", {}, { enabled: selectSwarm }).data
    ?.length
    ? true
    : false;
  const { mutateAsync: create, isPending: createPending } = useWrite(
    `Create${type}`,
  );
  const { mutateAsync: copy, isPending: copyPending } = useWrite(`Copy${type}`);
  const [swarmId, setSwarmId] = useState("");
  const [serverId, setServerId] = useState("");
  const [builderId, setBuilderId] = useState("");
  const [templateId, setTemplateId] = useState("");
  const [name, setName] = useState(_name);

  const typeDisplay =
    type === "ResourceSync" ? "resource-sync" : type.toLowerCase();

  const config: Types._PartialDeploymentConfig | Types._PartialRepoConfig =
    type === "Deployment"
      ? {
          swarm_id: _swarmId ?? swarmId,
          server_id: _serverId ?? serverId,
          image: buildId
            ? { type: "Build", params: { build_id: buildId } }
            : { type: "Image", params: { image: "" } },
        }
      : type === "Stack"
        ? { swarm_id: _swarmId ?? swarmId, server_id: _serverId ?? serverId }
        : type === "Repo"
          ? {
              server_id: _serverId ?? serverId,
              builder_id: _builderId ?? builderId,
            }
          : type === "Build"
            ? { builder_id: _builderId ?? builderId }
            : {};

  const onConfirm = async () => {
    if (!name.trim()) {
      notifications.show({ message: "Name cannot be empty", color: "red" });
    }
    try {
      const result = templateId
        ? await copy({ name: name.trim(), id: templateId })
        : await create({ name: name.trim(), config });
      const resourceId = result._id?.$oid;
      if (resourceId) {
        nav(`/${usableResourcePath(type)}/${resourceId}`);
      }
    } catch (error) {
      notifications.show({ message: "Unexpected error.", color: "red" });
    }
  };

  return (
    <CreateModal
      entityType={readableType ?? type}
      onConfirm={onConfirm}
      disabled={false}
      loading={createPending || copyPending}
      openShiftKeyListener="N"
      configSection={() => (
        <>
          <TextInput
            autoFocus
            placeholder={`${typeDisplay}-name`}
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => {
              if (!name) {
                return;
              }
              if (e.key === "Enter") {
                onConfirm();
              }
            }}
            error={!name.trim() && "Enter name"}
          />
        </>
      )}
    />
  );
}
