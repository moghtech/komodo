import { Checkbox, Code, Stack, TextInput } from "@mantine/core";
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
  const placeholder = value?.shell_mode
    ? "  # Add a command to run.\n  "
    : "  # Add multiple commands on new lines. Supports comments and escaped newlines.\n  ";
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
      <Checkbox
        label={
          <>
            Run in shell mode (
            <Code fz="sm" p="0">
              sh -c
            </Code>
            ){" "}
          </>
        }
        checked={value?.shell_mode ?? false}
        onChange={(e) =>
          set({
            ...(value || {}),
            shell_mode: e.currentTarget.checked,
          })
        }
        disabled={disabled}
      />
      <MonacoEditor
        value={value?.command || placeholder}
        language="shell"
        onValueChange={(command) => set({ ...(value || {}), command })}
        readOnly={disabled}
      />
    </Stack>
  );
}
