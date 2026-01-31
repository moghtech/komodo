import { Group, Stack, Text, TextInput } from "@mantine/core";
import { Types } from "komodo_client";
import { MonacoEditor } from "@/components/monaco";

export interface SystemCommandProps {
  value?: Types.SystemCommand;
  disabled: boolean;
  set: (value: Types.SystemCommand) => void;
}

export default function SystemCommand({
  value,
  disabled,
  set,
}: SystemCommandProps) {
  return (
    <Stack>
      <Group>
        <Text c="dimmed">Path:</Text>
        <TextInput
          placeholder="Command working directory"
          value={value?.path}
          w={{ base: 200, lg: 300 }}
          onChange={(e) => set({ ...(value || {}), path: e.target.value })}
          disabled={disabled}
        />
      </Group>
      <MonacoEditor
        value={
          value?.command ||
          "  # Add multiple commands on new lines. Supports comments.\n  "
        }
        language="shell"
        onValueChange={(command) => set({ ...(value || {}), command })}
        readOnly={disabled}
      />
    </Stack>
  );
}
