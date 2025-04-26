import { Section } from "@components/layouts";
import { ReactNode, useEffect, useMemo, useRef, useState } from "react";
import { useXTerm, UseXTermProps } from "react-xtermjs";
import { komodo_client, useLocalStorage, useRead, useWrite } from "@lib/hooks";
import { useTheme } from "@ui/theme";
import { ITheme } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { Card, CardContent, CardHeader } from "@ui/card";
import { Badge } from "@ui/badge";
import { Button } from "@ui/button";
import { Loader2, Plus, RefreshCcw, X } from "lucide-react";
import { cn } from "@lib/utils";

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
      shell: "bash",
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
            <ServerTerminal
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

const ServerTerminal = ({
  server,
  terminal,
  selected,
  _reconnect,
}: {
  server: string;
  terminal: string;
  selected: boolean;
  _reconnect: boolean;
}) => {
  const { theme: __theme } = useTheme();
  const _theme =
    __theme === "system"
      ? window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light"
      : __theme;
  const theme = _theme === "dark" ? DARK_THEME : LIGHT_THEME;
  const wsRef = useRef<WebSocket | null>(null);
  const fitRef = useRef<FitAddon>(new FitAddon());

  const resize = () => {
    fitRef.current.fit();
    if (term) {
      if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
        const json = JSON.stringify({
          rows: term.rows,
          cols: term.cols,
        });
        const buf = new Uint8Array(json.length + 1);
        buf[0] = 0xff; // resize prefix
        for (let i = 0; i < json.length; i++) buf[i + 1] = json.charCodeAt(i);
        wsRef.current.send(buf);
      }
      term.focus();
    }
  };

  const onStdin = (data: string) => {
    // This is data user writes to stdin
    if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) return;

    const buf = new Uint8Array(data.length + 1);
    buf[0] = 0x00; // data prefix
    for (let i = 0; i < data.length; i++) buf[i + 1] = data.charCodeAt(i);
    wsRef.current.send(buf);
  };

  useEffect(resize, [selected]);

  const params: UseXTermProps = useMemo(
    () => ({
      options: {
        convertEol: false,
        cursorBlink: true,
        cursorStyle: "block",
        fontFamily: "monospace",
        scrollback: 5000,
        // This is handled in ws on_message handler
        scrollOnUserInput: false,
        theme,
      },
      listeners: {
        onResize: resize,
        onData: onStdin,
      },
      addons: [fitRef.current],
    }),
    [theme]
  );

  const { instance: term, ref: termRef } = useXTerm(params);

  const viewport = (term as any)?._core?.viewport?._viewportElement as
    | HTMLDivElement
    | undefined;

  useEffect(() => {
    if (!term || !viewport) return;

    let delta = 0;
    term.attachCustomWheelEventHandler((e) => {
      e.preventDefault();
      // This is used to make touchpad and mousewheel more similar
      delta += Math.sign(e.deltaY) * Math.sqrt(Math.abs(e.deltaY)) * 20;
      return false;
    });
    const int = setInterval(() => {
      if (Math.abs(delta) < 1) return;
      viewport.scrollTop += delta;
      delta = 0;
    }, 100);
    return () => clearInterval(int);
  }, [term, termRef.current]);

  useEffect(() => {
    if (!selected || !term) return;

    term.clear();

    let debounce = -1;

    const ws = komodo_client().connect_terminal({
      query: {
        server,
        terminal,
      },
      on_login: () => {
        // console.log("logged in terminal");
      },
      on_open: resize,
      on_message: (e) => {
        term.write(new Uint8Array(e.data as ArrayBuffer), () => {
          if (viewport) {
            viewport.scrollTop = viewport.scrollHeight - viewport.clientHeight;
          }
          clearTimeout(debounce);
          debounce = setTimeout(() => {
            if (!viewport) return;
            viewport.scrollTop = viewport.scrollHeight - viewport.clientHeight;
          }, 500);
        });
      },
      on_close: () => {
        term.writeln("\r\n\x1b[33m[connection closed]\x1b[0m");
      },
    });

    wsRef.current = ws;

    return () => {
      ws.close();
      wsRef.current = null;
    };
  }, [term, viewport, selected, _reconnect]);

  return (
    <div
      ref={termRef}
      className={cn("w-full h-[65vh]", selected ? "" : "hidden")}
    />
  );
};

const LIGHT_THEME: ITheme = {
  background: "#f7f8f9",
  foreground: "#24292e",
  cursor: "#24292e",
  selectionBackground: "#c8d9fa",
};

const DARK_THEME: ITheme = {
  background: "#151b25",
  foreground: "#f6f8fa",
  cursor: "#ffffff",
  selectionBackground: "#6e778a",
};

const next_terminal_name = (terminal_names: string[]) => {
  for (let i = 1; i <= terminal_names.length + 1; i++) {
    const name = `Terminal ${i}`;
    if (!terminal_names.includes(name)) {
      return name;
    }
  }
  // This shouldn't happen
  return `Terminal -1`;
};
