import { MonacoDiffEditor, MonacoLanguage } from "mogh_monaco";
import { useState } from "react";
import { useDisclosure } from "@mantine/hooks";
import { Box, Button, Group, Modal, Stack, Text } from "@mantine/core";
import { useCtrlKeyListener, useKeyListener } from "mogh_ui";
import { fmtSnakeCaseToUpperSpaceCase } from "mogh_ui";
import { ICONS } from "@/lib/icons";
import { deepCompare, envToText } from "@/lib/utils";
import { ShowHideButton } from "mogh_ui";

export default function ConfirmUpdate<T>({
  original,
  update,
  onConfirm,
  loading,
  disabled,
  language,
  fileContentsLanguage,
  fullWidth,
  openKeyListener = true,
  confirmKeyListener = true,
}: {
  original: T;
  update: Partial<T>;
  onConfirm: () => Promise<unknown>;
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
      <Modal
        title={<Text size="xl">Confirm Update</Text>}
        opened={opened}
        onClose={close}
        size="auto"
        styles={{ content: { overflowY: "hidden" } }}
      >
        <Stack
          gap="xl"
          w={1400}
          maw={{
            base: "calc(100vw - 100px)",
            xs: "calc(100vw - 150px)",
            sm: "calc(100vw - 200px)",
            md: "calc(100vw - 250px)",
          }}
          my="lg"
          style={{ overflowY: "hidden" }}
        >
          <Stack
            mah="min(calc(100vh - 300px), 800px)"
            style={{ overflowY: "auto" }}
          >
            {Object.entries(update)
              .filter(([key, val]) => !deepCompare((original as any)[key], val))
              .map(([key, val], i) => (
                <ConfirmUpdateItem
                  key={i}
                  _key={key as any}
                  val={val as any}
                  previous={original}
                  language={language}
                  fileContentsLanguage={fileContentsLanguage}
                />
              ))}
          </Stack>
          <Group justify="flex-end">
            <Button
              leftSection={<ICONS.Save size="1rem" />}
              onClick={(e) => {
                e.stopPropagation();
                handleConfirm();
              }}
              w={{ base: "100%", xs: 200 }}
              loading={loading}
            >
              Save
            </Button>
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
      className="bordered-light"
      bdrs="md"
    >
      <Group justify="space-between">
        <Text c="Neutral">{fmtSnakeCaseToUpperSpaceCase(_key as string)}</Text>
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
              readOnly
            />
          ) : (
            <Box component="pre" mih={0}>
              <Text component="span" c="Critical">
                {prev_val || "None"}
              </Text>{" "}
              <Text component="span" c="dimmed">
                {"->"}
              </Text>{" "}
              <Text component="span" c="Good">
                {val || "None"}
              </Text>
            </Box>
          )}
        </>
      )}
    </Stack>
  );
}
