import { Section } from "@components/layouts";
import { ReactNode, useState } from "react";
import { useLocalStorage, useRead, useWrite } from "@lib/hooks";
import { Card, CardContent, CardHeader } from "@ui/card";
import { Badge } from "@ui/badge";
import { Button } from "@ui/button";
import { Loader2, Plus, RefreshCcw, X } from "lucide-react";
import { Terminal } from "@components/terminal";
import { Popover, PopoverContent, PopoverTrigger } from "@ui/popover";
import {
  Command,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@ui/command";
import { filterBySplit } from "@lib/utils";
import { useServer } from ".";

export const ServerTerminals = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther?: ReactNode;
}) => {
  const { data: terminals, refetch: refetchTerminals } = useRead(
    "ListTerminals",
    {
      server: id,
      fresh: true,
    },
    {
      refetchInterval: 5000,
    }
  );
  const { mutateAsync: create_terminal, isPending: create_pending } =
    useWrite("CreateTerminal");
  const { mutateAsync: delete_terminal } = useWrite("DeleteTerminal");
  const [_selected, setSelected] = useLocalStorage<{
    selected: string | undefined;
  }>(`server-${id}-selected-terminal-v1`, { selected: undefined });
  const terminals_disabled = useServer(id)?.info.terminals_disabled ?? true;

  const selected = _selected.selected ?? terminals?.[0]?.name;

  const [_reconnect, _setReconnect] = useState(false);
  const triggerReconnect = () => _setReconnect((r) => !r);

  const create = async (command: string) => {
    if (!terminals || terminals_disabled) return;
    const name = next_terminal_name(
      command,
      terminals.map((t) => t.name)
    );
    await create_terminal({
      server: id,
      name,
      command,
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
        <CardHeader className="flex flex-row gap-4 items-center justify-between flex-wrap">
          <div className="flex gap-4 items-center flex-wrap">
            {terminals?.map(({ name: terminal, stored_size_kb }) => (
              <Badge
                key={terminal}
                variant={terminal === selected ? "default" : "secondary"}
                className="w-fit min-w-[150px] px-2 py-1 cursor-pointer flex gap-4 justify-between"
                onClick={() => setSelected({ selected: terminal })}
              >
                <div className="text-sm w-full flex gap-1 items-center justify-between">
                  {terminal}
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
                    await delete_terminal({ server: id, terminal });
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
            {terminals && !terminals_disabled && (
              <NewTerminal create={create} pending={create_pending} />
            )}
          </div>
          <Button
            className="flex items-center gap-2"
            variant="secondary"
            onClick={() => triggerReconnect()}
          >
            Reconnect
            <RefreshCcw className="w-4 h-4" />
          </Button>
        </CardHeader>
        <CardContent className="min-h-[65vh]">
          {terminals?.map(({ name: terminal }) => (
            <Terminal
              key={terminal}
              query={{ server: id, terminal }}
              selected={selected === terminal}
              _reconnect={_reconnect}
            />
          ))}
        </CardContent>
      </Card>
    </Section>
  );
};

const BASE_SHELLS = ["bash", "sh"];

const NewTerminal = ({
  create,
  pending,
}: {
  create: (shell: string) => Promise<void>;
  pending: boolean;
}) => {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [shells, setShells] = useLocalStorage("server-shells-v1", BASE_SHELLS);
  const filtered = filterBySplit(shells, search, (item) => item);
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
            placeholder="Enter shell"
            className="h-9"
            value={search}
            onValueChange={setSearch}
          />
          <CommandList>
            <CommandGroup>
              {filtered.map((shell) => (
                <CommandItem
                  key={shell}
                  onSelect={() => {
                    create(shell);
                    setOpen(false);
                  }}
                  className="flex items-center justify-between cursor-pointer"
                >
                  <div className="p-1">{shell}</div>
                  {!BASE_SHELLS.includes(shell) && (
                    <Button
                      variant="destructive"
                      onClick={(e) => {
                        e.stopPropagation();
                        setShells((shells) =>
                          shells.filter((s) => s !== shell)
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
                    setShells((shells) => [...shells, search]);
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

const next_terminal_name = (command: string, terminal_names: string[]) => {
  const shell = command.split(" ")[0];
  for (let i = 1; i <= terminal_names.length + 1; i++) {
    const name = i > 1 ? `${shell} ${i}` : shell;
    if (!terminal_names.includes(name)) {
      return name;
    }
  }
  return shell;
};
