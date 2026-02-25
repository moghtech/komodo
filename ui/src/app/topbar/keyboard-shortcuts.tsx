import { useSettingsView, useShiftKeyListener } from "@/lib/hooks";
import {
  ActionIcon,
  Divider,
  Group,
  Kbd,
  Modal,
  SimpleGrid,
  Text,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { Keyboard } from "lucide-react";
import { useNavigate } from "react-router-dom";

export default function KeyboardShortcuts() {
  const [opened, { open, close }] = useDisclosure();
  const nav = useNavigate();
  const [_, setSettingsView] = useSettingsView();

  useShiftKeyListener("H", () => nav("/"));
  useShiftKeyListener("G", () => nav("/servers"));
  useShiftKeyListener("Z", () => nav("/stacks"));
  useShiftKeyListener("D", () => nav("/deployments"));
  useShiftKeyListener("B", () => nav("/builds"));
  useShiftKeyListener("R", () => nav("/repos"));
  useShiftKeyListener("P", () => nav("/procedures"));
  useShiftKeyListener("X", () => nav("/terminals"));
  useShiftKeyListener("C", () => nav("/schedules"));
  useShiftKeyListener("V", () => {
    setSettingsView("Variables");
    nav("/settings");
  });

  return (
    <>
      <Modal
        opened={opened}
        onClose={close}
        title={<Text size="xl">Keyboard Shortcuts</Text>}
        size="lg"
      >
        <SimpleGrid cols={2} p="md" bg="accent.1">
          <KeyboardShortcut label="Save" keys={["Ctrl / Cmd", "Enter"]} />
          <KeyboardShortcut label="New" keys={["Shift", "N"]} />
          <KeyboardShortcut label="Go Home" keys={["Shift", "H"]} />

          <KeyboardShortcut label="Go to Servers" keys={["Shift", "G"]} />
          <KeyboardShortcut label="Go to Stacks" keys={["Shift", "Z"]} />
          <KeyboardShortcut label="Go to Deployments" keys={["Shift", "D"]} />
          <KeyboardShortcut label="Go to Builds" keys={["Shift", "B"]} />
          <KeyboardShortcut label="Go to Repos" keys={["Shift", "R"]} />
          <KeyboardShortcut label="Go to Procedures" keys={["Shift", "P"]} />
          <KeyboardShortcut label="Go to Variables" keys={["Shift", "V"]} />
          <KeyboardShortcut label="Go to Terminals" keys={["Shift", "X"]} />
          <KeyboardShortcut label="Go to Schedules" keys={["Shift", "C"]} />

          <KeyboardShortcut label="Search" keys={["Shift", "S"]} />
          <KeyboardShortcut label="Add Filter Tag" keys={["Shift", "T"]} />
          <KeyboardShortcut
            label="Clear Filter Tags"
            keys={["Shift", "C"]}
            divider={false}
          />
        </SimpleGrid>
      </Modal>

      <ActionIcon
        size="xl"
        variant="subtle"
        c="inherit"
        visibleFrom="lg"
        onClick={open}
      >
        <Keyboard size="1.3rem" />
      </ActionIcon>
    </>
  );
}

function KeyboardShortcut({
  label,
  keys,
  divider = true,
}: {
  label: string;
  keys: string[];
  divider?: boolean;
}) {
  return (
    <>
      <Text>{label}</Text>
      <Group gap="xs">
        {keys.map((key) => (
          <Kbd key={key}>{key}</Kbd>
        ))}
      </Group>

      {divider && <Divider style={{ gridColumn: "1 / -1" }} />}
    </>
  );
}
