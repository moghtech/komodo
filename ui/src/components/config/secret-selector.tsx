import { useSearchCombobox, useSettingsView } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { filterBySplit } from "@/lib/utils";
import { Button, Combobox, ComboboxProps } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { useNavigate } from "react-router-dom";

export interface SecretSelectorProps extends ComboboxProps {
  keys: string[];
  type: "Variable" | "Secret";
}

export default function SecretSelector({
  keys,
  type,
  disabled,
  ...comboboxProps
}: SecretSelectorProps) {
  const nav = useNavigate();
  const [_, setSettingsView] = useSettingsView();
  const { search, setSearch, combobox } = useSearchCombobox({
    disableSelectFirst: true,
  });

  const filtered = filterBySplit(keys, search, (item) => item).sort((a, b) => {
    if (a > b) {
      return 1;
    } else if (a < b) {
      return -1;
    } else {
      return 0;
    }
  });

  return (
    <Combobox
      store={combobox}
      disabled={disabled}
      onOptionSubmit={(variable) => {
        if (variable == "Show All") {
          setSettingsView("Variables");
          nav("/settings");
        } else if (variable) {
          navigator.clipboard.writeText("[[" + variable + "]]");
          notifications.show({ message: "Copied secret key" });
        }
      }}
      width={300}
      {...comboboxProps}
    >
      <Combobox.Target>
        <Button
          leftSection={<ICONS.Search size="1rem" />}
          disabled={disabled}
          onClick={() => combobox.toggleDropdown()}
        >
          {type}s
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
          {filtered.map((key) => (
            <Combobox.Option key={key} value={key}>
              {key}
            </Combobox.Option>
          ))}
          <Combobox.Option value="Show All">Show All</Combobox.Option>
        </Combobox.Options>
      </Combobox.Dropdown>
    </Combobox>
  );
}
