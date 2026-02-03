import { MonacoDiffEditor, MonacoLanguage } from "@/components/monaco";
import { colorByIntention } from "@/lib/color";
import { fmtSnakeCaseToUpperSpaceCase } from "@/lib/formatting";
import { useCtrlKeyListener, useKeyListener } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { envToText } from "@/lib/utils";
import ShowHideButton from "@/ui/show-hide-button";
import { Box, Button, Group, Modal, Stack, Text } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useState } from "react";
import ConfirmButton from "../confirm-button";

export default function ConfirmUpdate<T>({
  previous,
  content,
  onConfirm,
  loading,
  disabled,
  language,
  fileContentsLanguage,
  fullWidth,
  openKeyListener = true,
  confirmKeyListener = true,
}: {
  previous: T;
  content: Partial<T>;
  onConfirm: () => Promise<void>;
  loading?: boolean;
  disabled: boolean;
  language?: MonacoLanguage;
  fileContentsLanguage?: MonacoLanguage;
  fullWidth?: boolean;
  openKeyListener?: boolean;
  confirmKeyListener?: boolean;
}) {
  const [opened, { open, close }] = useDisclosure();

  const handleConfirm = async () => {
    await onConfirm();
    close();
  };

  useKeyListener("Enter", () => {
    if (!opened || !confirmKeyListener) {
      return;
    }
    handleConfirm();
  });

  useCtrlKeyListener("Enter", () => {
    if (opened || !openKeyListener) {
      return;
    }
    open();
  });

  return (
    <>
      <Modal title="Confirm Update" opened={opened} onClose={close} size="auto">
        <Stack gap="xl" w={1400} maw="95vw" my="lg">
          <Stack>
            {Object.entries(content).map(([key, val], i) => (
              <ConfirmUpdateItem
                key={i}
                _key={key as any}
                val={val as any}
                previous={previous}
                language={language}
                fileContentsLanguage={fileContentsLanguage}
              />
            ))}
          </Stack>
          <Group justify="flex-end">
            <ConfirmButton
              icon={<ICONS.Save size="1rem" />}
              onClick={handleConfirm}
              loading={loading}
            >
              Save
            </ConfirmButton>
          </Group>
        </Stack>
      </Modal>

      <Button
        leftSection={<ICONS.Save size="1rem" />}
        onClick={(e) => {
          e.stopPropagation();
          open();
        }}
        disabled={disabled}
        w={fullWidth ? undefined : 100}
        fullWidth={fullWidth}
      >
        Save
      </Button>
    </>
  );
}

function ConfirmUpdateItem<T>({
  _key,
  val: _val,
  previous,
  language,
  fileContentsLanguage,
}: {
  _key: keyof T;
  val: T[keyof T];
  previous: T;
  language?: MonacoLanguage;
  fileContentsLanguage?: MonacoLanguage;
}) {
  const [show, setShow] = useState(true);
  const val =
    typeof _val === "string"
      ? _val
      : Array.isArray(_val)
        ? _val.length > 0 &&
          ["string", "number", "boolean"].includes(typeof _val[0])
          ? JSON.stringify(_val)
          : JSON.stringify(_val, null, 2)
        : JSON.stringify(_val, null, 2);
  const prev_val =
    typeof previous[_key] === "string"
      ? previous[_key]
      : _key === "environment" ||
          _key === "build_args" ||
          _key === "secret_args"
        ? (envToText(previous[_key] as any) ?? "") // For backward compat with 1.14
        : Array.isArray(previous[_key])
          ? previous[_key].length > 0 &&
            ["string", "number", "boolean"].includes(typeof previous[_key][0])
            ? JSON.stringify(previous[_key])
            : JSON.stringify(previous[_key], null, 2)
          : JSON.stringify(previous[_key], null, 2);
  const showDiff =
    val?.includes("\n") ||
    prev_val?.includes("\n") ||
    Math.max(val?.length ?? 0, prev_val?.length ?? 0) > 30;

  return (
    <Stack
      hidden={val === prev_val}
      gap="xs"
      p="xl"
      bd="1px solid var(--mantine-color-accent-border-0)"
      bdrs="md"
    >
      <Group justify="space-between">
        <Text c={colorByIntention("Neutral")}>
          {fmtSnakeCaseToUpperSpaceCase(_key as string)}
        </Text>
        <ShowHideButton show={show} setShow={setShow} />
      </Group>
      {show && (
        <>
          {showDiff ? (
            <MonacoDiffEditor
              original={prev_val}
              modified={val}
              language={
                language ??
                (["environment", "build_args", "secret_args"].includes(
                  _key as string,
                )
                  ? "key_value"
                  : _key === "file_contents"
                    ? fileContentsLanguage
                    : "json")
              }
            />
          ) : (
            <Box component="pre" mih={0}>
              <Text component="span" c={colorByIntention("Critical")}>
                {prev_val || "None"}
              </Text>{" "}
              <Text component="span" c="dimmed">
                {"->"}
              </Text>{" "}
              <Text component="span" c={colorByIntention("Good")}>
                {val || "None"}
              </Text>
            </Box>
          )}
        </>
      )}
    </Stack>
  );
}
