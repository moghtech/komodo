import { CopyButton } from "mogh_ui";
import { useManageAuth, useUserInvalidate } from "@/lib/hooks";
import {
  Button,
  Center,
  Flex,
  Loader,
  Modal,
  Text,
  TextInput,
} from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { Types } from "komodo_client";
import { Check, RotateCcwKey, Trash } from "lucide-react";
import { useState } from "react";
import { ConfirmModal } from "mogh_ui";

export const EnrollTotp = ({ user }: { user: Types.User }) => {
  const userInvalidate = useUserInvalidate();
  const [open, setOpen] = useState(false);
  const [submitted, setSubmitted] = useState<{ uri: string; png: string }>();
  const [confirm, setConfirm] = useState("");
  const [recovery, setRecovery] = useState<string[] | undefined>(undefined);

  const { mutate: beginEnrollment } = useManageAuth("BeginTotpEnrollment", {
    onSuccess: ({ uri, png }) => setSubmitted({ uri, png }),
  });

  const { mutate: confirmEnrollment, isPending: confirmPending } =
    useManageAuth("ConfirmTotpEnrollment", {
      onSuccess: ({ recovery_codes }) => {
        setRecovery(recovery_codes);
        userInvalidate();
      },
    });

  const { mutateAsync: unenroll, isPending: unenrollPending } = useManageAuth(
    "UnenrollTotp",
    {
      onSuccess: () => {
        userInvalidate();
        notifications.show({
          message: "Unenrolled in TOTP 2FA.",
          color: "green",
        });
      },
    },
  );

  const onOpenChange = (open: boolean) => {
    setOpen(open);
    if (open) {
      beginEnrollment({});
    } else {
      setSubmitted(undefined);
      setRecovery(undefined);
    }
  };

  return (
    <>
      {!user.totp?.confirmed_at && !user.passkey?.created_at && (
        <>
          <Modal opened={open} onClose={() => onOpenChange(false)}>
            {recovery && (
              <Flex direction="column" gap="lg">
                <Text size="lg">Save recovery keys</Text>
                <Flex direction="column" gap="sm">
                  {recovery.map((code) => (
                    <TextInput key={code} w={200} value={code} disabled />
                  ))}
                </Flex>
                <CopyButton content={recovery.join("\n")} />
              </Flex>
            )}
            {!recovery && submitted && (
              <Flex direction="column" gap="lg">
                <Text size="lg">
                  Scan this QR code with your authenticator app, and enter the 6
                  digit code below.
                </Text>
                <Center>
                  <img
                    width={250}
                    height={250}
                    src={"data:image/png;base64," + submitted.png}
                    alt="QRCode"
                  />
                </Center>
                <Flex align="center" justify="space-between" gap="sm">
                  URI
                  <TextInput w={250} value={submitted.uri} disabled />
                  <CopyButton content={submitted.uri} />
                </Flex>
                <Flex align="center" justify="space-between">
                  Confirm Code
                  <TextInput
                    w={250}
                    value={confirm}
                    onChange={(e) => setConfirm(e.target.value)}
                    autoFocus
                  />
                </Flex>
                <Flex justify="flex-end">
                  <Button
                    onClick={() => confirmEnrollment({ code: confirm })}
                    disabled={confirm.length !== 6 || confirmPending}
                    leftSection={<Check size="1rem" />}
                    loading={confirmPending}
                  >
                    Confirm
                  </Button>
                </Flex>
              </Flex>
            )}
            {!recovery && !submitted && (
              <Center>
                <Loader />
              </Center>
            )}
          </Modal>
          <Button
            leftSection={<RotateCcwKey size="1rem" />}
            variant="default"
            onClick={() => onOpenChange(true)}
            w={220}
          >
            Enroll TOTP 2FA
          </Button>
        </>
      )}
      {!!user.totp?.confirmed_at && (
        <ConfirmModal
          confirmText="Unenroll"
          icon={<Trash size="1rem" />}
          loading={unenrollPending}
          onConfirm={() => unenroll({})}
          targetProps={{ c: "bw", w: 220 }}
          confirmProps={{ variant: "filled", color: "red" }}
        >
          Unenroll TOTP 2FA
        </ConfirmModal>
      )}
    </>
  );
};
