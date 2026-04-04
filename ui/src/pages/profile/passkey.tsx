import { useUserInvalidate } from "@/lib/hooks";
import { Button } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { MoghAuth, Types } from "komodo_client";
import { Fingerprint, Trash } from "lucide-react";
import { ConfirmModal, useManageAuth } from "mogh_ui";

export const EnrollPasskey = ({ user }: { user: Types.User }) => {
  const userInvalidate = useUserInvalidate();

  const { mutateAsync: unenroll, isPending: unenrollPending } = useManageAuth(
    "UnenrollPasskey",
    {
      onSuccess: () => {
        userInvalidate();
        notifications.show({
          message: "Unenrolled in passkey 2FA",
          color: "green",
        });
      },
    },
  );

  const { mutate: confirmEnrollment } = useManageAuth(
    "ConfirmPasskeyEnrollment",
    {
      onSuccess: () => {
        userInvalidate();
        notifications.show({
          message: "Enrolled in passkey authentication",
          color: "green",
        });
      },
    },
  );

  const { mutate: beginEnrollment } = useManageAuth("BeginPasskeyEnrollment", {
    onSuccess: (challenge) => {
      navigator.credentials
        .create(MoghAuth.Passkey.prepareCreationChallengeResponse(challenge))
        .then((credential) => confirmEnrollment({ credential }))
        .catch((e) => {
          console.error(e);
          notifications.show({
            title: "Failed to create passkey",
            message: "See console for details",
            color: "red",
          });
        });
    },
  });

  return (
    <>
      {!user.passkey?.created_at && !user.totp?.confirmed_at && (
        <Button
          leftSection={<Fingerprint size="1rem" />}
          onClick={() => beginEnrollment({})}
          hidden={!!user?.passkey}
          w={220}
        >
          Enroll Passkey 2FA
        </Button>
      )}
      {!!user.passkey?.created_at && (
        <ConfirmModal
          confirmText="Unenroll"
          icon={<Trash size="1rem" />}
          loading={unenrollPending}
          onConfirm={() => unenroll({})}
          targetProps={{ c: "bw", w: 220 }}
          confirmProps={{ variant: "filled", color: "red" }}
        >
          Unenroll Passkey 2FA
        </ConfirmModal>
      )}
    </>
  );
};
