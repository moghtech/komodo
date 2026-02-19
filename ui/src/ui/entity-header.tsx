import { ColorIntention, hexColorByIntention } from "@/lib/color";
import { ICONS } from "@/theme/icons";
import { ActionIcon, Group, Stack, Text, TextInput } from "@mantine/core";
import { FC, ReactNode, useEffect, useState } from "react";

export interface EntityHeaderProps {
  name?: string;
  icon: FC<{ size?: string | number; color?: string }>;
  intent: ColorIntention;
  state?: ReactNode;
  status?: ReactNode;
  action?: ReactNode;
  onRename?: (name: string) => Promise<unknown>;
  renamePending?: boolean;
}

export default function EntityHeader({
  name,
  icon: Icon,
  intent,
  state,
  status,
  action,
  onRename: _onRename,
  renamePending,
}: EntityHeaderProps) {
  const [editingName, setEditingName] = useState(false);
  const [newName, setNewName] = useState(name);
  useEffect(() => {
    setNewName(name);
  }, [name]);
  const color = hexColorByIntention(intent);
  const background = color ? color + "25" : undefined;
  const onRename =
    _onRename &&
    (() => {
      if (!newName) return;
      if (name === newName) {
        setEditingName(false);
      } else {
        _onRename(newName).then(() => setEditingName(false));
      }
    });
  return (
    <Group
      justify="space-between"
      gap="lg"
      px="xl"
      py="md"
      style={{
        background,
        borderTopLeftRadius: "var(--mantine-radius-md)",
        borderTopRightRadius: "var(--mantine-radius-md)",
      }}
    >
      <Group gap="lg">
        <Icon size="2rem" color={color} />
        <Stack gap="0">
          {name && !onRename && (
            <Text fz="h1" fw="bolder">
              {name}
            </Text>
          )}
          {onRename && (
            <Group
              mb={editingName ? "xs" : undefined}
              gap="xs"
              onClick={editingName ? undefined : () => setEditingName(true)}
              style={{ cursor: editingName ? undefined : "pointer" }}
            >
              {!editingName && (
                <>
                  <Text fz="h1" fw="bolder">
                    {name}
                  </Text>
                  <ActionIcon title="Rename">
                    <ICONS.Edit size="1rem" />
                  </ActionIcon>
                </>
              )}
              {editingName && (
                <>
                  <TextInput
                    placeholder="Enter name"
                    value={newName}
                    onChange={(e) => setNewName(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && onRename()}
                    disabled={renamePending}
                    size="lg"
                    autoFocus
                  />
                  <ActionIcon
                    variant="filled"
                    color="blue"
                    onClick={onRename}
                    loading={renamePending}
                  >
                    <ICONS.Save size="1rem" />
                  </ActionIcon>
                  <ActionIcon
                    variant="filled"
                    color="red"
                    onClick={() => {
                      setNewName(name);
                      setEditingName(false);
                    }}
                  >
                    <ICONS.Clear size="1rem" />
                  </ActionIcon>
                </>
              )}
            </Group>
          )}
          <Group fz="md" tt="uppercase" mt="-8" gap="sm">
            <Text c={color} fw="600">
              {state}
            </Text>
            <Text c="dimmed">{status}</Text>
          </Group>
        </Stack>
      </Group>
      {action}
    </Group>
  );
}
