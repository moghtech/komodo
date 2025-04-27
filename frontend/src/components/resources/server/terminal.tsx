import { Section } from "@components/layouts";
import { ReactNode, useEffect, useState } from "react";
import { useLocalStorage, useRead, useWrite } from "@lib/hooks";
import { Card, CardContent, CardHeader } from "@ui/card";
import { Badge } from "@ui/badge";
import { Button } from "@ui/button";
import { Loader2, Plus, RefreshCcw, X } from "lucide-react";
import { Terminal } from "@components/terminal";

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

  const selected =
    _selected.selected ??
    terminals?.[0]?.name ??
    next_terminal_name(terminals?.map((t) => t.name) ?? []);

  const [_reconnect, _setReconnect] = useState(false);
  const triggerReconnect = () => _setReconnect((r) => !r);

  const create = async () => {
    if (!terminals) return;
    const name = next_terminal_name(terminals.map((t) => t.name));
    await create_terminal({
      server: id,
      name,
      command: "bash",
    });
    refetchTerminals();
    setTimeout(() => {
      setSelected({
        selected: name,
      });
    }, 100);
  };

  useEffect(() => {
    if (terminals && terminals.length === 0) {
      create();
    }
  }, [terminals]);

  return (
    <Section titleOther={titleOther}>
      <Card>
        <CardHeader className="flex flex-row gap-4 items-center justify-between">
          <div className="flex gap-4">
            {terminals?.map(({ name: terminal }) => (
              <Badge
                key={terminal}
                variant={terminal === selected ? "default" : "secondary"}
                className="w-fit min-w-[150px] px-2 py-1 cursor-pointer flex gap-4 justify-between"
                onClick={() => setSelected({ selected: terminal })}
              >
                {terminal}
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
            {terminals && (
              <Button
                className="flex items-center gap-2"
                variant="outline"
                onClick={create}
                disabled={create_pending}
              >
                New Terminal
                {create_pending ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                  <Plus className="w-4 h-4" />
                )}
              </Button>
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
              server={id}
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

const next_terminal_name = (terminal_names: string[]) => {
  for (let i = 1; i <= terminal_names.length + 1; i++) {
    const name = `terminal ${i}`;
    if (!terminal_names.includes(name)) {
      return name;
    }
  }
  // This shouldn't happen
  return `terminal -1`;
};
