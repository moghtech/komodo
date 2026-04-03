import { Types } from "komodo_client";
import { PROCEDURE_EXECUTIONS } from "./executions";
import { filterBySplit } from "@/lib/utils";
import { Button, ButtonProps, Combobox, ComboboxProps } from "@mantine/core";
import { ChevronsUpDown } from "lucide-react";
import { fmtUpperCamelcase } from "mogh_ui";
import { ICONS } from "@/lib/icons";
import { useSearchCombobox } from "@/lib/hooks";

export interface ProcedureExecutionSelectorProps extends ComboboxProps {
  type: Types.Execution["type"];
  onSelect: (type: Types.Execution["type"]) => void;
  disabled: boolean;
  targetProps?: ButtonProps;
}

export default function ProcedureExecutionSelector({
  type,
  onSelect,
  disabled,
  onOptionSubmit,
  position = "bottom-start",
  targetProps,
  ...comboboxProps
}: ProcedureExecutionSelectorProps) {
  const executionTypes = Object.keys(PROCEDURE_EXECUTIONS).filter(
    (c) => !["None"].includes(c),
  );

  const { search, setSearch, combobox } = useSearchCombobox();

  const filtered = filterBySplit(executionTypes, search, (item) => item);

  return (
    <Combobox
      store={combobox}
      width={300}
      position={position}
      onOptionSubmit={(type, props) => {
        onSelect?.(type as Types.Execution["type"]);
        onOptionSubmit?.(type, props);
        combobox.closeDropdown();
      }}
      {...comboboxProps}
    >
      <Combobox.Target>
        <Button
          justify="space-between"
          rightSection={<ChevronsUpDown size="1rem" />}
          onClick={() => combobox.toggleDropdown()}
          disabled={disabled}
          w="fit-content"
          maw={{ base: 200, lg: 300 }}
          {...targetProps}
        >
          {fmtUpperCamelcase(type)}
        </Button>
      </Combobox.Target>
      <Combobox.Dropdown>
        <Combobox.Search
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          leftSection={<ICONS.Search size="1rem" style={{ marginRight: 6 }} />}
          placeholder="Search"
        />
        <Combobox.Options mah={224} style={{ overflowY: "auto" }}>
          {!search && <Combobox.Option value="None">None</Combobox.Option>}
          {filtered.map((type) => (
            <Combobox.Option key={type} value={type}>
              {fmtUpperCamelcase(type)}
            </Combobox.Option>
          ))}
          {filtered.length === 0 && (
            <Combobox.Empty>No results.</Combobox.Empty>
          )}
        </Combobox.Options>
      </Combobox.Dropdown>
    </Combobox>
  );
}
