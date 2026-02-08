import { useWrite } from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import { ICONS } from "@/theme/icons";
import {
  Button,
  ButtonProps,
  Combobox,
  ComboboxProps,
  Divider,
  Group,
  Text,
  useCombobox,
} from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import { useEffect, useState } from "react";

export interface NewTerminalProps extends ComboboxProps {
  target: Types.TerminalTarget;
  existingTerminals: string[] | undefined;
  refetchTerminals: () => void;
  setSelected: (value: { selected: string | undefined }) => void;
  services?: string[];
  targetProps?: ButtonProps;
}

const BASE_COMMANDS = ["sh", "bash"];

export default function NewTerminal({
  target,
  existingTerminals,
  refetchTerminals,
  setSelected,
  services,
  position = "bottom-start",
  targetProps,
  ...comboboxProps
}: NewTerminalProps) {
  const [service, setService] = useState<string | undefined>(undefined);
  const { mutateAsync: createTerminal } = useWrite("CreateTerminal");

  const [search, setSearch] = useState("");
  const combobox = useCombobox({
    onDropdownOpen: () => {
      combobox.focusSearchInput();
    },
    onDropdownClose: () => {
      combobox.resetSelectedOption();
      combobox.focusTarget();
      setSearch("");
    },
  });
  useEffect(() => {
    combobox.selectFirstOption();
  }, [search]);

  const create = async (command: string | undefined, isServer: boolean) => {
    if (!existingTerminals) return;
    const name = nextTerminalName(command, service, existingTerminals);
    await createTerminal({
      target: service
        ? { ...target, params: { ...target.params, service } as any }
        : target,
      name,
      command,
      mode:
        !isServer && !command
          ? Types.ContainerTerminalMode.Attach
          : Types.ContainerTerminalMode.Exec,
    });
    refetchTerminals();
    setTimeout(() => {
      setSelected({
        selected: name,
      });
    }, 100);
  };

  const isServer = target.type === "Server";

  const [commands, setCommands] = useLocalStorage({
    key: isServer ? "server-commands-v2" : "container-commands-v2",
    defaultValue: isServer ? BASE_COMMANDS : [...BASE_COMMANDS, "attach"],
  });
  const filtered = filterBySplit(commands, search, (item) => item);

  return (
    <Combobox
      store={combobox}
      width={300}
      position={position}
      onOptionSubmit={(command) => {
        if (!!services && !service) {
          setService(command);
        } else {
          create(
            command === "Default" || (!isServer && command === "attach")
              ? undefined
              : command === "Custom"
                ? search
                : command,
            isServer,
          ).then(() => {
            combobox.closeDropdown();
            setService(undefined);
          });
        }
      }}
      onClose={() => setService(undefined)}
      {...comboboxProps}
    >
      <Combobox.Target>
        <Button
          leftSection={<ICONS.Create size="1rem" />}
          onClick={() => combobox.toggleDropdown()}
          {...targetProps}
        >
          New
        </Button>
      </Combobox.Target>
      <Combobox.Dropdown>
        <Combobox.Search
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          leftSection={<ICONS.Search size="1rem" style={{ marginRight: 6 }} />}
          placeholder={
            !!services && !service ? "Select Service" : "Select Command"
          }
        />
        <Combobox.Options mah={224} style={{ overflowY: "auto" }}>
          {!!services &&
            !service &&
            services.map((service) => (
              <Combobox.Option key={service} value={service}>
                <Text>{service}</Text>
              </Combobox.Option>
            ))}
          {(!services || !!service) && (
            <>
              {isServer && !search && (
                <Combobox.Option value="Default">Default</Combobox.Option>
              )}
              {filtered.map((command) => (
                <Combobox.Option key={command} value={command}>
                  <Text>{command}</Text>
                </Combobox.Option>
              ))}

              <Divider />

              <Combobox.Option
                value="Custom"
                disabled={!search || commands.includes(search)}
                onSelect={() => setCommands((c) => [...c, search])}
              >
                <Group justify="center" gap="xs">
                  <ICONS.Create size="1rem" />
                  Custom
                </Group>
              </Combobox.Option>
            </>
          )}
        </Combobox.Options>
      </Combobox.Dropdown>
    </Combobox>
  );
}

function nextTerminalName(
  __command: string | undefined,
  service: string | undefined,
  existingTerminals: string[],
) {
  const _command = !__command ? "attach" : __command.split(" ")[0];
  const command = `${service ? service + " " : ""}${_command}`;
  for (let i = 1; i <= existingTerminals.length + 1; i++) {
    const name = i > 1 ? `${command} ${i}` : command;
    if (!existingTerminals.includes(name)) {
      return name;
    }
  }
  return command;
}
