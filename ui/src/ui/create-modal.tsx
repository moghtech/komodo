import { useShiftKeyListener } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { Button, ButtonProps, Group, Modal, Stack, Text } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { ReactNode, useEffect } from "react";

export interface CreateModalProps extends ButtonProps {
  entityType: string;
  configSection: (close: () => void) => ReactNode;
  loading?: boolean;
  onConfirm: () => Promise<boolean>;
  onOpenChange?: (opened: boolean) => void;
  configureLabel?: string;
  openShiftKeyListener?: string;
  children?: ReactNode;
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
  leftSection,
  children,
  ...targetProps
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
        <Stack gap="0.2rem">
          <Text c="dimmed" mb="md">
            Enter {configureLabel} for the new {entityType}.
          </Text>

          {configSection(close)}

          <Group justify="flex-end" mt="lg">
            <Button
              onClick={async () =>
                onConfirm().then((success) => success && close())
              }
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
        leftSection={leftSection || <ICONS.Create size="1rem" />}
        onClick={open}
        w={{ base: "100%", xs: "fit-content" }}
        {...targetProps}
      >
        {children ?? <>New {entityType}</>}
      </Button>
    </>
  );
}
