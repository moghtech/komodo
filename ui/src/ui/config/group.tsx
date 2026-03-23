import { Fragment } from "react";
import { ConfigFieldArgs, ConfigGroupArgs } from ".";
import { ConfigInput, ConfigSwitch } from "./item";
import { Group, Stack } from "@mantine/core";
import { ICONS } from "@/theme/icons";

export default function ConfigGroup<T>({
  config,
  update,
  setUpdate,
  disabled,
  fields,
}: {
  config: T;
  update: Partial<T>;
  setUpdate: (update: Partial<T>) => void;
  disabled: boolean;
  fields: ConfigGroupArgs<T>["fields"];
}) {
  return (
    <Stack gap="xl">
      {Object.entries(fields).map(([key, field]) => {
        const value =
          (update as { [key: string]: unknown })[key] ??
          (config as { [key: string]: unknown })[key];
        if (typeof field === "function") {
          return <Fragment key={key}>{field(value, setUpdate)}</Fragment>;
        } else if (typeof field === "object" || field === true) {
          const args =
            typeof field === "object" ? (field as ConfigFieldArgs) : undefined;

          if (args?.hidden) {
            return null;
          }

          switch (typeof value) {
            case "string":
              return (
                <ConfigInput
                  key={key}
                  label={args?.label ?? key}
                  value={value}
                  onValueChange={(value) =>
                    setUpdate({ [key]: value } as Partial<T>)
                  }
                  disabled={args?.disabled || disabled}
                  placeholder={args?.placeholder}
                  description={args?.description}
                  boldLabel={args?.boldLabel}
                />
              );

            case "number":
              return (
                <ConfigInput
                  key={key}
                  label={args?.label ?? key}
                  value={Number(value)}
                  onValueChange={(value) =>
                    setUpdate({ [key]: Number(value) } as Partial<T>)
                  }
                  disabled={args?.disabled || disabled}
                  placeholder={args?.placeholder}
                  description={args?.description}
                  boldLabel={args?.boldLabel}
                />
              );

            case "boolean":
              return (
                <ConfigSwitch
                  key={key}
                  label={args?.label ?? key}
                  value={value}
                  onCheckedChange={(value) =>
                    setUpdate({ [key]: value } as Partial<T>)
                  }
                  disabled={args?.disabled || disabled}
                  description={args?.description}
                  boldLabel={args?.boldLabel}
                />
              );

            default:
              return (
                <Group>
                  Config '{args?.label ?? key}': <ICONS.Unknown size="1rem" />
                </Group>
              );
          }
        } else {
          return <Fragment key={key} />;
        }
      })}
    </Stack>
  );
}
