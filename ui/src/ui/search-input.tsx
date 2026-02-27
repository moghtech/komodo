import { ICONS } from "@/theme/icons";
import { TextInput, TextInputProps } from "@mantine/core";

export interface SearchInputProps extends TextInputProps {
  onSearch?: (search: string) => void;
}

export default function SearchInput({
  onSearch,
  onChange,
  ...inputProps
}: SearchInputProps) {
  return (
    <TextInput
      placeholder="search..."
      leftSection={<ICONS.Search size="0.8rem" />}
      w={{ base: "100%", xs: 220 }}
      onChange={(e) => {
        onChange?.(e);
        onSearch?.(e.target.value);
      }}
      {...inputProps}
    />
  );
}
