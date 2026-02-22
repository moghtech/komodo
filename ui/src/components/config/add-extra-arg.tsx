import { useRead, useSearchCombobox } from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import { Button, Combobox } from "@mantine/core";
import { ICONS } from "@/theme/icons";

export interface AddExtraArgProps {
  type: "Deployment" | "Build" | "Stack" | "StackBuild";
  onSelect: (suggestion: string) => void;
  disabled?: boolean;
}

export default function AddExtraArg({
  type,
  onSelect,
  disabled,
}: AddExtraArgProps) {
  const suggestions = useRead(`ListCommon${type}ExtraArgs`, {}).data ?? [];
  const { search, setSearch, combobox } = useSearchCombobox();

  if (suggestions.length === 0) {
    return (
      <Button
        leftSection={<ICONS.Add size="1rem" />}
        onClick={() => onSelect("")}
        disabled={disabled}
        w={200}
      >
        Add Extra Arg
      </Button>
    );
  }

  const filtered = filterBySplit(suggestions, search, (item) => item);

  return (
    <Combobox
      store={combobox}
      disabled={disabled}
      onOptionSubmit={(suggestion) => {
        onSelect(suggestion);
        combobox.closeDropdown();
      }}
      width={250}
      position="bottom-start"
    >
      <Combobox.Target>
        <Button
          onClick={() => combobox.openDropdown()}
          leftSection={<ICONS.Add size="1rem" />}
          disabled={disabled}
          w={200}
        >
          Add Extra Arg
        </Button>
      </Combobox.Target>

      <Combobox.Dropdown>
        <Combobox.Search
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          leftSection={<ICONS.Search size="1rem" />}
          placeholder="Search"
          styles={{
            section: {
              marginRight: 4,
            },
          }}
        />
        <Combobox.Options mah={224} style={{ overflowY: "auto" }}>
          <Combobox.Option value={search}>Custom Extra Arg</Combobox.Option>
          {filtered.map((suggestion) => (
            <Combobox.Option key={suggestion} value={suggestion}>
              {suggestion}
            </Combobox.Option>
          ))}
        </Combobox.Options>
      </Combobox.Dropdown>
    </Combobox>
  );
}
