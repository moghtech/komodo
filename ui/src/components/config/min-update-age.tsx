import { ConfigItem } from "mogh_ui";
import { Group, TextInput } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { useEffect, useState } from "react";

const MIN_UPDATE_AGE_ERROR =
  "Minimum update age must be a whole number between 0 and 4294967295";

function parseMinUpdateAge(input: string) {
  const trimmed = input.trim();
  if (!/^\d+$/.test(trimmed)) return undefined;
  const hours = Number(trimmed);
  return hours <= 0xffff_ffff ? hours : undefined;
}

export function MinUpdateAge({
  arg,
  set,
  disabled,
  description,
}: {
  arg: number;
  set: (input: { min_update_age_hours: number }) => void;
  disabled: boolean;
  description?: string;
}) {
  const [input, setInput] = useState(arg.toString());
  useEffect(() => {
    setInput(arg.toString());
  }, [arg]);
  const error =
    parseMinUpdateAge(input) === undefined ? MIN_UPDATE_AGE_ERROR : undefined;
  return (
    <ConfigItem
      label="Minimum Update Age"
      description={
        description ??
        "Require the image to have been in the registry this long before Auto Update deploys it. 0 disables the check."
      }
    >
      <Group gap="xs">
        <TextInput
          w={100}
          placeholder="0"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onBlur={(e) => {
            const hours = parseMinUpdateAge(e.target.value);
            if (hours !== undefined) {
              set({ min_update_age_hours: hours });
            } else {
              notifications.show({
                message: MIN_UPDATE_AGE_ERROR,
                color: "red",
              });
            }
          }}
          error={error}
          disabled={disabled}
        />
        hours
      </Group>
    </ConfigItem>
  );
}
