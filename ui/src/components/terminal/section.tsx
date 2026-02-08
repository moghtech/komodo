import { useRead, useWrite } from "@/lib/hooks";
import Section, { SectionProps } from "@/ui/section";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import { useState } from "react";
import { ActionIcon, Button, Center, Group, Stack, Text } from "@mantine/core";
import TargetTerminal from "./target";
import { ICONS } from "@/theme/icons";
import NewTerminal from "./new";

export interface TerminalSectionProps extends Omit<SectionProps, "children"> {
  target: Types.TerminalTarget;
  /** Use with Stack, where each service is a potential target */
  services?: string[];
}

export default function TerminalSection({
  target,
  services,
  ...props
}: TerminalSectionProps) {
  const { data: terminals, refetch: refetchTerminals } = useRead(
    "ListTerminals",
    {
      target,
    },
    {
      refetchInterval: 5000,
    },
  );

  const { mutateAsync: deleteTerminal } = useWrite("DeleteTerminal");

  const [_selected, setSelected] = useLocalStorage<{
    selected: string | undefined;
  }>({
    key: `${JSON.stringify(target)}-selected-terminal-v1`,
    defaultValue: { selected: undefined },
  });

  const selected = _selected.selected ?? terminals?.[0]?.name;

  const [_reconnect, _setReconnect] = useState(false);
  const triggerReconnect = () => _setReconnect((r) => !r);

  return (
    <Section {...props}>
      <Group>
        {terminals?.map(({ name: terminal, target, stored_size_kb }) => {
          const isSelected = terminal === selected;
          return (
            <Group
              key={terminal}
              onClick={() => setSelected({ selected: terminal })}
              style={{ cursor: "pointer" }}
              className="accent-hover"
              bg={isSelected ? "accent.9" : undefined}
              justify="space-between"
              px="md"
              py="0.4rem"
              bd="1px solid var(--mantine-color-accent-border-7)"
              bdrs="sm"
            >
              <Group gap="xs">
                <Text fw={isSelected ? "bold" : undefined}>{terminal}</Text>
                <Text c="dimmed">{stored_size_kb.toFixed()} KiB</Text>
              </Group>
              <ActionIcon
                color="red"
                onClick={async (e) => {
                  e.stopPropagation();
                  await deleteTerminal({ target, terminal });
                  refetchTerminals();
                  if (selected === terminal) {
                    setSelected({ selected: undefined });
                  }
                }}
              >
                <ICONS.Delete size="1rem" />
              </ActionIcon>
            </Group>
          );
        })}

        <NewTerminal
          target={target}
          existingTerminals={terminals?.map((t) => t.name)}
          refetchTerminals={refetchTerminals}
          setSelected={setSelected}
        />

        {terminals?.[0] ? (
          <Button
            leftSection={<ICONS.Refresh size="1rem" />}
            onClick={() => triggerReconnect()}
          >
            Reconnect
          </Button>
        ) : null}
      </Group>
      {terminals?.map(({ name: terminal, target }) => (
        <TargetTerminal
          key={terminal}
          terminal={terminal}
          target={target}
          selected={selected === terminal}
          _reconnect={_reconnect}
        />
      ))}
      {terminals && !terminals.length && (
        <Center h="20vh">
          <Stack align="center" justify="center" gap="0">
            <Text fz="h2">No Terminals</Text>
            <Text c="dimmed">
              Create a new terminal using the <b>New</b> button.
            </Text>
          </Stack>
        </Center>
      )}
    </Section>
  );
}
