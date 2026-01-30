import {
  Button,
  ButtonProps,
  Group,
  Modal,
  Stack,
  Text,
  TextInput,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { ReactNode, useState } from "react";
import ConfirmButton from "./confirm-button";
import { notifications } from "@mantine/notifications";

export interface ConfirmModalProps {
  children?: string;
  icon: ReactNode;
  disabled?: boolean;
  /** User must enter this text to confirm */
  confirmText: string;
  title?: string;
  onConfirm?: () => Promise<unknown>;
  loading?: boolean;
  additional?: ReactNode;
  topAdditonal?: ReactNode;
  /** Converts into ConfirmButton */
  disableModal?: boolean;
  targetProps?: ButtonProps;
}

export default function ConfirmModal({
  children,
  icon,
  disabled,
  confirmText,
  title,
  onConfirm,
  loading,
  additional,
  topAdditonal,
  disableModal,
  targetProps,
}: ConfirmModalProps) {
  const [opened, { open, close }] = useDisclosure();
  const [input, setInput] = useState("");

  if (disableModal) {
    return (
      <ConfirmButton
        icon={icon}
        onClick={() => (onConfirm ? onConfirm().then(() => close()) : close())}
        loading={loading}
        disabled={disabled}
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
        title={title ?? `Confirm ${children}`}
      >
        <Stack>
          {topAdditonal}

          <Text
            onClick={() => {
              navigator.clipboard.writeText(confirmText);
              notifications.show({
                message: `Copied "${confirmText}" to clipboard.`,
              });
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

          <Group justify="flex-end">
            <ConfirmButton
              icon={icon}
              disabled={disabled || input !== confirmText}
              onClick={() => {
                onConfirm ? onConfirm().then(() => close()) : close();
              }}
              loading={loading}
            >
              {children}
            </ConfirmButton>
          </Group>
        </Stack>
      </Modal>

      <Button
        onClick={open}
        leftSection={icon}
        disabled={disabled}
        {...targetProps}
      >
        {children}
      </Button>
    </>
  );
}
