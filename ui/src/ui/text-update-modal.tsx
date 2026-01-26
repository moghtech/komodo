import { Button, Group, Modal, Stack, Text, Textarea } from "@mantine/core";
import { CheckCircle } from "lucide-react";
import { ReactNode, useEffect, useState } from "react";

export default function TextUpdateModal({
  title,
  titleRight,
  value: _value = "",
  onUpdate,
  placeholder,
  disabled,
  open,
  setOpen,
}: {
  title: string;
  titleRight?: ReactNode;
  value: string | undefined;
  onUpdate: (value: string) => void;
  placeholder?: string;
  confirmButton?: boolean;
  disabled?: boolean;
  open: boolean;
  setOpen: (open: boolean) => void;
}) {
  const [value, setValue] = useState(_value);
  useEffect(() => setValue(_value), [_value]);
  const onClick = () => {
    onUpdate(value);
    setOpen(false);
  };

  return (
    <Modal
      opened={open}
      onClose={() => setOpen(false)}
      bdrs="md"
      size="lg"
      title={
        titleRight ? (
          <Group>
            <Text fz="h2">{title}</Text>
            {titleRight}
          </Group>
        ) : (
          <Text fz="h2">{title}</Text>
        )
      }
    >
      <Stack>
        <Textarea
          value={value}
          onChange={(e) => setValue(e.target.value)}
          placeholder={placeholder}
          disabled={disabled}
          resize="vertical"
        />

        {!disabled && (
          <Group justify="end" w="100%">
            <Button
              variant="light"
              c="inherit"
              leftSection={<CheckCircle size="1rem" />}
              onClick={onClick}
            >
              Update
            </Button>
          </Group>
        )}
      </Stack>
    </Modal>
  );
}
