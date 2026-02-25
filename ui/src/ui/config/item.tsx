import { fmtSnakeCaseToUpperSpaceCase } from "@/lib/formatting";
import { ICONS } from "@/theme/icons";
import EnableSwitch from "@/ui/enable-switch";
import InputList, { InputListProps } from "@/ui/input-list";
import {
  Button,
  createPolymorphicComponent,
  Group,
  Stack,
  StackProps,
  SwitchProps,
  Text,
  TextInput,
  TextInputProps,
} from "@mantine/core";
import { forwardRef, ReactNode } from "react";

// https://mantine.dev/guides/polymorphic/#create-your-own-polymorphic-components

export interface ConfigItemProps extends StackProps {
  label?: ReactNode;
  boldLabel?: boolean;
  labelExtra?: ReactNode;
  description?: ReactNode;
  children?: ReactNode;
}

export const ConfigItem = createPolymorphicComponent<"div", ConfigItemProps>(
  forwardRef<HTMLDivElement, ConfigItemProps>(
    (
      { label, boldLabel, labelExtra, description, children, ...props },
      ref,
    ) => {
      const labelDescription = (label || description) && (
        <Stack gap="0">
          {typeof label === "string" && (
            <Text fw={boldLabel ? "bold" : undefined} fz="h3">
              {fmtSnakeCaseToUpperSpaceCase(label)}
            </Text>
          )}
          {label && typeof label !== "string" && label}
          {description && (
            <Text c="dimmed" fz="sm">
              {description}
            </Text>
          )}
        </Stack>
      );
      return (
        <Stack {...props} ref={ref}>
          {labelExtra ? (
            <Group>
              {labelDescription}
              {labelExtra}
            </Group>
          ) : (
            labelDescription
          )}
          {children}
        </Stack>
      );
    },
  ),
);

export function ConfigInput({
  value,
  disabled,
  placeholder,
  onChange,
  onValueChange,
  onBlur,
  inputLeft,
  inputRight,
  inputProps,
  email,
  ...itemProps
}: {
  value: string | number | undefined;
  disabled?: boolean;
  placeholder?: string;
  onValueChange?: (value: string) => void;
  onBlur?: (value: string) => void;
  inputLeft?: ReactNode;
  inputRight?: ReactNode;
  inputProps?: TextInputProps;
  email?: boolean;
} & Omit<ConfigItemProps, "children">) {
  const inputNode = (
    <TextInput
      w={{ base: "85%", lg: 400 }}
      value={value}
      placeholder={placeholder}
      disabled={disabled}
      type={typeof value === "number" ? "number" : email ? "email" : undefined}
      onChange={(e) => {
        onChange?.(e);
        onValueChange?.(e.target.value);
      }}
      onBlur={(e) => onBlur?.(e.target.value)}
      {...inputProps}
    />
  );
  return (
    <ConfigItem {...itemProps}>
      {inputLeft || inputRight ? (
        <Group>
          {inputLeft}
          {inputNode}
          {inputRight}
        </Group>
      ) : (
        inputNode
      )}
    </ConfigItem>
  );
}

export function ConfigSwitch({
  value,
  disabled,
  onChange,
  switchProps,
  ...itemProps
}: {
  value: boolean | undefined;
  disabled: boolean;
  onChange: (value: boolean) => void;
  switchProps?: SwitchProps;
} & Omit<Omit<ConfigItemProps, "children">, "onChange">) {
  return (
    <ConfigItem {...itemProps}>
      <EnableSwitch
        checked={value}
        onCheckedChange={onChange}
        disabled={disabled}
        {...switchProps}
      />
    </ConfigItem>
  );
}

export function ConfigList<T>({
  addLabel,
  label,
  boldLabel,
  description,
  ...inputListProps
}: { label?: string; addLabel?: string } & InputListProps<T> &
  Omit<ConfigItemProps, "children">) {
  return (
    <ConfigItem label={label} boldLabel={boldLabel} description={description}>
      <InputList
        inputProps={{ w: { base: "85%", lg: 400 } }}
        {...inputListProps}
      />
      {!inputListProps.disabled && (
        <Button
          leftSection={<ICONS.Create size="1rem" />}
          onClick={() =>
            inputListProps.set({
              [inputListProps.field]: [...inputListProps.values, ""],
            } as Partial<T>)
          }
          w={{ base: "85%", lg: 400 }}
          disabled={inputListProps.disabled}
        >
          {addLabel ??
            ("Add " + label?.endsWith("s") ? label?.slice(0, -1) : label)}
        </Button>
      )}
    </ConfigItem>
  );
}
