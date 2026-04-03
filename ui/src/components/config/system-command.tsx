import { Stack, TextInput } from "@mantine/core";
import { Types } from "komodo_client";
import { MonacoEditor } from "mogh_ui";

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
      <TextInput
        label="Path"
        placeholder="Command working directory"
        value={value?.path}
        w={{ base: "85%", sm: 300 }}
        onChange={(e) => set({ ...(value || {}), path: e.target.value })}
        disabled={disabled}
      />
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
