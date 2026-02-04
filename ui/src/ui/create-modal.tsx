import { useShiftKeyListener } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { Button, Group, Modal, Stack, Text } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { ReactNode, useEffect } from "react";

export interface CreateModalProps {
  entityType: string;
  configSection: (close: () => void) => ReactNode;
  disabled: boolean;
  loading: boolean;
  onConfirm: () => Promise<unknown>;
  onOpenChange?: (opened: boolean) => void;
  configureLabel?: string;
  openShiftKeyListener?: string;
}

export default function CreateModal({
  entityType,
  configSection,
  disabled,
  loading,
  onConfirm,
  onOpenChange,
  configureLabel = "a unique name",
  openShiftKeyListener,
}: CreateModalProps) {
  const [opened, { open, close }] = useDisclosure();
  useEffect(() => onOpenChange?.(opened), [opened]);
  useShiftKeyListener(
    openShiftKeyListener ?? "___",
    () => openShiftKeyListener && !opened && open(),
  );
  return (
    <>
      <Modal
        opened={opened}
        onClose={close}
        title={`New ${entityType}`}
        withCloseButton={false}
        trapFocus
      >
        <Stack>
          <Text c="dimmed">
            Enter {configureLabel} for the new {entityType}.
          </Text>

          {configSection(close)}

          <Group justify="flex-end">
            <Button
              onClick={() => {
                onConfirm().then(close);
              }}
              loading={loading}
              disabled={disabled}
            >
              Create
            </Button>
          </Group>
        </Stack>
      </Modal>

      <Button
        variant="default"
        leftSection={<ICONS.Create size="1rem" />}
        onClick={open}
      >
        New {entityType}
      </Button>
    </>
  );
}
