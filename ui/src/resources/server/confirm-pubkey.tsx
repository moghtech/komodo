import { Group, Text } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { CircleOff } from "lucide-react";
import { usePermissions, useWrite } from "@/lib/hooks";
import ConfirmModal from "@/ui/confirm-modal";
import { ServerComponents } from ".";

export default function ConfirmServerPubkey({ id }: { id: string }) {
  const server = ServerComponents.useListItem(id);
  const { canWrite } = usePermissions({ type: "Server", id });
  const { mutateAsync: confirm, isPending } = useWrite(
    "UpdateServerPublicKey",
    {
      onSuccess: () => {
        notifications.show({
          message: "Confirmed Server public key",
          color: "green",
        });
      },
    },
  );

  if (!server?.info.attempted_public_key) return null;

  return (
    <ConfirmModal
      disabled={!canWrite}
      title="Confirm Public Key"
      confirmButtonText="Confirm"
      confirmText={server.name}
      icon={<CircleOff size="1rem" />}
      targetProps={{ color: "red" }}
      topAdditonal={
        <Group gap="xs">
          <Text c="dimmed">Public Key:</Text>
          {server.info.attempted_public_key}
        </Group>
      }
      additional={
        <Text>Note. May take a few moments for status to update.</Text>
      }
      onConfirm={() =>
        confirm({ server: id, public_key: server.info.attempted_public_key! })
      }
      loading={isPending}
    >
      Invalid Pubkey
    </ConfirmModal>
  );
}
