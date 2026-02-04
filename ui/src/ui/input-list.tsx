import { ICONS } from "@/theme/icons";
import { ActionIcon, TextInput, TextInputProps } from "@mantine/core";

export interface InputListProps<T> {
  field: keyof T;
  values: string[];
  disabled: boolean;
  set: (update: Partial<T>) => void;
  placeholder?: string;
  inputProps?: TextInputProps;
}

export default function InputList<T>({
  field,
  values,
  disabled,
  set,
  placeholder,
  inputProps,
}: InputListProps<T>) {
  return (
    <>
      {values.map((arg, i) => (
        <TextInput
          key={i}
          value={arg}
          onChange={(e) => {
            values[i] = e.target.value;
            set({ [field]: [...values] } as Partial<T>);
          }}
          disabled={disabled}
          w={{ base: 230, md: 400 }}
          rightSection={
            !disabled && (
              <ActionIcon
                color="red"
                onClick={() =>
                  set({
                    [field]: [...values.filter((_, idx) => idx !== i)],
                  } as Partial<T>)
                }
              >
                <ICONS.Remove size="1rem" />
              </ActionIcon>
            )
          }
          placeholder={placeholder}
          {...inputProps}
        />
      ))}
    </>
  );
}
