import { Section } from "@components/layouts";
import { komodo_client, useLocalStorage, useRead, useWrite } from "@lib/hooks";
import { Button } from "@ui/button";
import { Card, CardContent, CardHeader } from "@ui/card";
import { Loader2, Plus, RefreshCcw, SearchX, X } from "lucide-react";
import { ReactNode, useCallback, useState } from "react";
import { Terminal } from ".";
import { TerminalCallbacks, Types } from "komodo_client";
import { Badge } from "@ui/badge";
import { filterBySplit } from "@lib/utils";
import { Popover, PopoverContent, PopoverTrigger } from "@ui/popover";
import {
  Command,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@ui/command";

const BASE_COMMANDS = ["sh", "bash", "attach"];

export const ContainerTerminals = ({
  target,
  titleOther,
  services,
}: {
  target: Types.TerminalTarget;
  titleOther?: ReactNode;
  services?: string[];
}) => {
  const { data: terminals, refetch: refetchTerminals } = useRead(
    "ListTerminals",
    {
      target,
    },
    {
      refetchInterval: 5000,
    },
  );
  const { mutateAsync: create_terminal, isPending: create_pending } =
    useWrite("CreateTerminal");
  const { mutateAsync: delete_terminal } = useWrite("DeleteTerminal");
  const [_selected, setSelected] = useLocalStorage<{
    selected: string | undefined;
  }>(`${JSON.stringify(target)}-selected-terminal-v1`, { selected: undefined });

  const selected = _selected.selected ?? terminals?.[0]?.name;

  const [_reconnect, _setReconnect] = useState(false);
  const triggerReconnect = () => _setReconnect((r) => !r);

  const create = async (command: string | undefined, service?: string) => {
    if (!terminals) return;
    const name = next_terminal_name(
      command,
      terminals.map((t) => t.name),
    );
    await create_terminal({
      target: service
        ? { ...target, params: { ...target.params, service } as any }
        : target,
      name,
      command,
      mode: !command
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
  return (
    <Section titleOther={titleOther}>
      <Card>
        <CardHeader>
          <div className="flex gap-4 items-center flex-wrap">
            {terminals?.map(({ name: terminal, target, stored_size_kb }) => (
              <Badge
                key={terminal}
                variant={terminal === selected ? "default" : "secondary"}
                className="w-fit min-w-[150px] px-2 py-1 cursor-pointer flex gap-4 justify-between items-center"
                onClick={() => setSelected({ selected: terminal })}
              >
                <div className="text-sm w-full flex gap-4 items-center justify-between">
                  <div className="flex gap-1 items-center">
                    {services && (
                      <div className="text-muted-foreground">
                        {(target.params as any)?.service} -
                      </div>
                    )}
                    {terminal}
                  </div>
                  {/* <div className="min-w-[20px] max-w-[70px] text-xs text-muted-foreground text-nowrap whitespace-nowrap overflow-hidden overflow-ellipsis">
                    {command}
                  </div> */}
                  <div className="text-muted-foreground text-xs">
                    {stored_size_kb.toFixed()} KiB
                  </div>
                </div>
                <Button
                  className="p-1 h-fit"
                  variant="destructive"
                  onClick={async (e) => {
                    e.stopPropagation();
                    await delete_terminal({ target, terminal });
                    refetchTerminals();
                    if (selected === terminal) {
                      setSelected({ selected: undefined });
                    }
                  }}
                >
                  <X className="w-4 h-4" />
                </Button>
              </Badge>
            ))}
            {terminals && services ? (
              <NewServiceTerminal
                services={services}
                create={create}
                pending={create_pending}
              />
            ) : (
              <NewTerminal create={create} pending={create_pending} />
            )}
            <Button
              className="flex items-center gap-2 m-0"
              variant="secondary"
              onClick={() => triggerReconnect()}
            >
              Reconnect
              <RefreshCcw className="w-4 h-4" />
            </Button>
          </div>
        </CardHeader>
        <CardContent className="min-h-[65vh]">
          {terminals?.map(({ name: terminal, target }) => (
            <ContainerTerminal
              key={terminal}
              target={target}
              terminal={terminal}
              selected={selected === terminal}
              _reconnect={_reconnect}
            />
          ))}
        </CardContent>
      </Card>
    </Section>
  );
};

export const ContainerTerminal = ({
  terminal,
  target,
  selected,
  _reconnect,
}: {
  terminal: string;
  target: Types.TerminalTarget;
  selected: boolean;
  _reconnect: boolean;
}) => {
  const make_ws = useCallback(
    (callbacks: TerminalCallbacks) => {
      return komodo_client().connect_terminal({
        query: { target, terminal },
        ...callbacks,
      });
    },
    [target, terminal],
  );
  return (
    <Terminal make_ws={make_ws} selected={selected} _reconnect={_reconnect} />
  );
};

const NewTerminal = ({
  create,
  pending,
}: {
  create: (command: string | undefined) => Promise<void>;
  pending: boolean;
}) => {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [commands, setCommands] = useLocalStorage(
    "container-commands-v1",
    BASE_COMMANDS,
  );
  const filtered = filterBySplit(commands, search, (item) => item);
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          className="flex items-center gap-2"
          disabled={pending}
        >
          New Terminal
          {pending ? (
            <Loader2 className="w-4 h-4 animate-spin" />
          ) : (
            <Plus className="w-4 h-4" />
          )}
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[200px] max-h-[300px] p-0" align="start">
        <Command shouldFilter={false}>
          <CommandInput
            placeholder="Enter command"
            className="h-9"
            value={search}
            onValueChange={setSearch}
          />
          <CommandList>
            <CommandGroup>
              {filtered.map((command) => (
                <CommandItem
                  key={command}
                  onSelect={() => {
                    create(command === "attach" ? undefined : command);
                    setOpen(false);
                  }}
                  className="flex items-center justify-between cursor-pointer"
                >
                  <div className="p-1">{command}</div>
                  {!BASE_COMMANDS.includes(command) && (
                    <Button
                      variant="destructive"
                      onClick={(e) => {
                        e.stopPropagation();
                        setCommands((commands) =>
                          commands.filter((s) => s !== command),
                        );
                      }}
                      className="p-1 h-fit"
                    >
                      <X className="w-4 h-4" />
                    </Button>
                  )}
                </CommandItem>
              ))}
              {filtered.length === 0 && (
                <CommandItem
                  onSelect={() => {
                    setCommands((commands) => [...commands, search]);
                    create(search);
                    setOpen(false);
                  }}
                  className="flex items-center justify-between cursor-pointer"
                >
                  <div className="p-1">{search}</div>
                  <Plus className="w-4 h-4" />
                </CommandItem>
              )}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
};

const NewServiceTerminal = ({
  create,
  pending,
  services,
}: {
  create: (command: string | undefined, service: string) => Promise<void>;
  pending: boolean;
  services: string[];
}) => {
  const [service, setService] = useState<string | undefined>(undefined);
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [commands, setCommands] = useLocalStorage(
    "container-commands-v1",
    BASE_COMMANDS,
  );
  const filteredServices = filterBySplit(services, search, (item) => item);
  const filteredCommands = filterBySplit(commands, search, (item) => item);
  return (
    <Popover
      open={open}
      onOpenChange={(open) => {
        if (!open) setService(undefined);
        setOpen(open);
      }}
    >
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          className="flex items-center gap-2"
          disabled={pending}
        >
          New Terminal
          {pending ? (
            <Loader2 className="w-4 h-4 animate-spin" />
          ) : (
            <Plus className="w-4 h-4" />
          )}
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[200px] max-h-[300px] p-0" align="start">
        <Command shouldFilter={false}>
          <CommandInput
            placeholder={service ? "Enter command" : "Select service"}
            className="h-9"
            value={search}
            onValueChange={setSearch}
          />
          <CommandList>
            <CommandGroup>
              {!service && (
                <>
                  {filteredServices.map((service) => (
                    <CommandItem
                      key={service}
                      onSelect={() => setService(service)}
                      className="flex items-center justify-between cursor-pointer"
                    >
                      <div className="p-1">{service}</div>
                    </CommandItem>
                  ))}
                  {filteredServices.length === 0 && (
                    <CommandItem className="flex items-center justify-between cursor-pointer text-muted-foreground">
                      <div className="p-1">No results</div>
                      <SearchX className="w-4 h-4" />
                    </CommandItem>
                  )}
                </>
              )}
              {service && (
                <>
                  {filteredCommands.map((command) => (
                    <CommandItem
                      key={command}
                      onSelect={() => {
                        create(
                          command === "attach" ? undefined : command,
                          service,
                        );
                        setService(undefined);
                        setOpen(false);
                      }}
                      className="flex items-center justify-between cursor-pointer"
                    >
                      <div className="p-1">{command}</div>
                      {!BASE_COMMANDS.includes(command) && (
                        <Button
                          variant="destructive"
                          onClick={(e) => {
                            e.stopPropagation();
                            setCommands((commands) =>
                              commands.filter((s) => s !== command),
                            );
                          }}
                          className="p-1 h-fit"
                        >
                          <X className="w-4 h-4" />
                        </Button>
                      )}
                    </CommandItem>
                  ))}
                  {filteredCommands.length === 0 && (
                    <CommandItem
                      onSelect={() => {
                        setCommands((commands) => [...commands, search]);
                        create(search, service);
                        setService(undefined);
                        setOpen(false);
                      }}
                      className="flex items-center justify-between cursor-pointer"
                    >
                      <div className="p-1">{search}</div>
                      <Plus className="w-4 h-4" />
                    </CommandItem>
                  )}
                </>
              )}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
};

const next_terminal_name = (
  _command: string | undefined,
  terminal_names: string[],
) => {
  const command = !_command ? "attach" : _command.split(" ")[0];
  for (let i = 1; i <= terminal_names.length + 1; i++) {
    const name = i > 1 ? `${command} ${i}` : command;
    if (!terminal_names.includes(name)) {
      return name;
    }
  }
  return command;
};
