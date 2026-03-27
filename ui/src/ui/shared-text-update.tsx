import {
  Button,
  Group,
  Modal,
  ModalProps,
  Stack,
  Text,
  Textarea,
} from "@mantine/core";
import { CheckCircle } from "lucide-react";
import { Dispatch, ReactNode, SetStateAction, useState } from "react";

export interface SharedTextUpdateData {
  title: string;
  value: string;
  placeholder: string;
  onUpdate: (value: string) => void;
  titleRight?: ReactNode;
}

export function useSharedTextUpdateData() {
  return useState<false | SharedTextUpdateData>(false);
}

export interface SharedTextUpdateProps extends Omit<
  Omit<ModalProps, "opened">,
  "onClose"
> {
  data: false | SharedTextUpdateData;
  setData: Dispatch<SetStateAction<false | SharedTextUpdateData>>;
  disabled?: boolean;
}

export default function SharedTextUpdate({
  data,
  setData,
  disabled,
  ...modalProps
}: SharedTextUpdateProps) {
  return (
    <Modal
      title={
        data &&
        (data.titleRight ? (
          <Group>
            <Text fz="h2">{data.title}</Text>
            {data.titleRight}
          </Group>
        ) : (
          <Text fz="h2">{data.title}</Text>
        ))
      }
      opened={!!data}
      onClose={() => setData(false)}
      size="xl"
      {...modalProps}
    >
      {data && (
        <Stack>
          <Textarea
            value={data.value}
            placeholder={data.placeholder}
            onChange={(e) =>
              setData((data) =>
                data ? { ...data, value: e.target.value } : false,
              )
            }
            disabled={disabled}
            resize="vertical"
            styles={{ input: { minHeight: 200 } }}
          />

          {!disabled && (
            <Group justify="end" w="100%">
              <Button
                leftSection={<CheckCircle size="1rem" />}
                onClick={() => {
                  data.onUpdate(data.value);
                  setData(false);
                }}
              >
                Update
              </Button>
            </Group>
          )}
        </Stack>
      )}
    </Modal>
  );
}
