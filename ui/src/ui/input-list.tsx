import { ICONS } from "@/lib/icons";
import { ActionIcon, TextInput, TextInputProps } from "@mantine/core";

export interface InputListProps<T> {
  field: keyof T;
  values: string[];
  disabled: boolean;
  set: (update: Partial<T>) => void;
  inputProps?: TextInputProps;
}

export default function InputList<T>({
  field,
  values,
  disabled,
  set,
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
                variant="light"
                color="red"
                c="inherit"
                onClick={() =>
                  set({
                    [field]: [...values.filter((_, idx) => idx !== i)],
                  } as Partial<T>)
                }
              >
                <ICONS.Delete size="1rem" />
              </ActionIcon>
            )
          }
          {...inputProps}
        />
      ))}
    </>
  );
}
