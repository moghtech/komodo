import { MonacoEditor, MonacoLanguage } from "mogh_monaco";
import {
  Button,
  Group,
  Modal,
  ModalProps,
  Stack,
  Text,
  Textarea,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { CheckCircle } from "lucide-react";
import { ReactNode, useEffect, useState } from "react";

export interface TextUpdateModalProps extends Omit<
  Omit<ModalProps, "opened">,
  "onClose"
> {
  title: string;
  titleRight?: ReactNode;
  value: string | undefined;
  onUpdate: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  target?: (open: () => void) => ReactNode;
  /* If passed, use monaco editor instead */
  useMonaco?: boolean;
  monacoLanguage?: MonacoLanguage;
}

function defaultTarget(open: () => void, value: string) {
  return (
    <Button variant="outline" onClick={open}>
      {value}
    </Button>
  );
}

export default function TextUpdateModal({
  title,
  titleRight,
  value: _value = "",
  onUpdate,
  placeholder,
  disabled,
  target,
  useMonaco,
  monacoLanguage,
  ...modalProps
}: TextUpdateModalProps) {
  const [opened, { open, close }] = useDisclosure();
  const [value, setValue] = useState(_value);
  useEffect(() => setValue(_value), [_value]);
  const onClick = () => {
    onUpdate(value);
    close();
  };

  return (
    <>
      <Modal
        opened={opened}
        onClose={close}
        size="xl"
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
        {...modalProps}
      >
        <Stack>
          {useMonaco ? (
            <MonacoEditor
              value={value}
              onValueChange={setValue}
              readOnly={disabled}
              language={monacoLanguage}
            />
          ) : (
            <Textarea
              value={value}
              onChange={(e) => setValue(e.target.value)}
              placeholder={placeholder}
              disabled={disabled}
              resize="vertical"
              styles={{ input: { minHeight: 200 } }}
            />
          )}

          {!disabled && (
            <Group justify="end" w="100%">
              <Button
                leftSection={<CheckCircle size="1rem" />}
                onClick={onClick}
              >
                Update
              </Button>
            </Group>
          )}
        </Stack>
      </Modal>

      {target ? target(open) : defaultTarget(open, value)}
    </>
  );
}
