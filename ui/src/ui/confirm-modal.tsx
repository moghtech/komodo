import {
  Button,
  ButtonProps,
  Group,
  Loader,
  Modal,
  ModalProps,
  Stack,
  Text,
  TextInput,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { ReactNode, useState } from "react";
import ConfirmButton from "./confirm-button";
import { sendCopyNotification } from "@/lib/utils";

export interface ConfirmModalProps extends Omit<
  Omit<Omit<ModalProps, "opened">, "onClose">,
  "onClick"
> {
  children?: ReactNode;
  icon?: ReactNode;
  disabled?: boolean;
  /** User must enter this text to confirm */
  confirmText: string;
  title?: ReactNode;
  confirmButtonContent?: ReactNode;
  onConfirm?: () => Promise<unknown>;
  loading?: boolean;
  additional?: ReactNode;
  topAdditonal?: ReactNode;
  /** Converts into ConfirmButton */
  disableModal?: boolean;
  targetProps?: ButtonProps;
  targetNoIcon?: boolean;
}

export default function ConfirmModal({
  children,
  icon,
  disabled,
  confirmText,
  title,
  confirmButtonContent,
  onConfirm,
  loading,
  additional,
  topAdditonal,
  disableModal,
  targetProps,
  targetNoIcon,
  ...modalProps
}: ConfirmModalProps) {
  const [opened, { open, close }] = useDisclosure();
  const [input, setInput] = useState("");

  if (disableModal) {
    return (
      <ConfirmButton
        icon={icon}
        onClick={onConfirm}
        disabled={disabled}
        loading={loading}
        {...targetProps}
      >
        {children}
      </ConfirmButton>
    );
  }

  return (
    <>
      <Modal
        opened={opened}
        onClose={close}
        title={
          <Text fz="h3">
            {title ?? (
              <>
                Confirm <b>{children}</b>
              </>
            )}
          </Text>
        }
        styles={{ content: { padding: "0.5rem" } }}
        size="lg"
        onClick={(e) => e.stopPropagation()}
        {...modalProps}
      >
        <Stack>
          {topAdditonal}

          <Text
            onClick={() => {
              navigator.clipboard.writeText(confirmText);
              sendCopyNotification();
            }}
            style={{ cursor: "pointer" }}
          >
            Please enter <b>{confirmText}</b> below to confirm this action.
            {(location.origin.startsWith("https") ||
              // For dev
              location.origin.startsWith("http://localhost:")) && (
              <Text fz="sm" c="dimmed">
                You may click the text in bold to copy it
              </Text>
            )}
          </Text>

          <TextInput
            value={input}
            onChange={(e) => setInput(e.target.value)}
            error={input === confirmText ? undefined : "Does not match"}
          />

          {additional}

          <Group justify="end">
            <Button
              justify="space-between"
              w={{ base: "100%", xs: 190 }}
              miw="fit-content"
              rightSection={
                loading ? <Loader color="white" size="1rem" /> : icon
              }
              disabled={loading || disabled || input !== confirmText}
              onClick={(e) => {
                e.stopPropagation();
                onConfirm ? onConfirm().then(() => close()) : close();
              }}
            >
              {confirmButtonContent ?? children}
            </Button>
          </Group>
        </Stack>
      </Modal>

      <Button
        onClick={(e) => {
          e.stopPropagation();
          open();
        }}
        justify="space-between"
        w={{ base: "100%", xs: 190 }}
        miw="fit-content"
        rightSection={
          targetNoIcon ? undefined : loading ? (
            <Loader color="white" size="1rem" />
          ) : (
            icon
          )
        }
        loading={targetNoIcon ? loading : undefined}
        disabled={disabled || loading}
        {...targetProps}
      >
        {children}
      </Button>
    </>
  );
}
