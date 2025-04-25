import { Section } from "@components/layouts";
import { ReactNode, useEffect, useMemo, useRef, useState } from "react";
import { FitAddon } from "@xterm/addon-fit";
import { useXTerm, UseXTermProps } from "react-xtermjs";
import { komodo_client, useLocalStorage, useRead, useWrite } from "@lib/hooks";
import { useTheme } from "@ui/theme";
import { ITheme } from "@xterm/xterm";
import { Card, CardContent, CardHeader } from "@ui/card";
import { Badge } from "@ui/badge";
import { Button } from "@ui/button";
import { X } from "lucide-react";
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
      refetchInterval: 3000,
    }
  );
  const { mutateAsync: delete_terminal } = useWrite("DeleteTerminal");
  const [_selected, setSelected] = useLocalStorage<{
    selected: string | undefined;
  }>(`server-${id}-selected-terminal-v1`, { selected: undefined });

  const selected =
    _selected.selected ?? terminals?.[0] ?? next_terminal_name(terminals ?? []);

  const [_reconnect, _setReconnect] = useState(false);
  const triggerReconnect = () => _setReconnect((r) => !r);

  return (
    <Section titleOther={titleOther}>
      <Card>
        <CardHeader className="flex">
          <div className="flex gap-4">
            {terminals?.map((terminal) => (
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
                  }}
                >
                  <X className="w-4 h-4" />
                </Button>
              </Badge>
            ))}
            {terminals && (
              <Button
                variant="outline"
                onClick={() => {
                  setSelected({ selected: next_terminal_name(terminals) });
                  setTimeout(() => refetchTerminals(), 1000);
                }}
              >
                New
              </Button>
            )}
          </div>
        </CardHeader>
        <CardContent>
          {terminals?.map((terminal) => (
            <ServerTerminal
              key={terminal}
              server={id}
              terminal={terminal}
              selected={selected === terminal}
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
}: {
  server: string;
  terminal: string;
  selected: boolean;
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
  // const [_reconnect, _setReconnect] = useState(false);
  // const triggerReconnect = () => _setReconnect((r) => !r);

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

  useEffect(resize, [selected]);

  const params: UseXTermProps = useMemo(
    () => ({
      options: {
        convertEol: true,
        cursorBlink: true,
        cursorStyle: "bar",
        cursorWidth: 2,
        fontFamily: "monospace",
        theme,
      },
      listeners: {
        onResize: ({ rows, cols }) => {
          if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN)
            return;

          const json = JSON.stringify({ rows, cols });
          const buf = new Uint8Array(json.length + 1);
          buf[0] = 0xff; // resize prefix
          for (let i = 0; i < json.length; i++) buf[i + 1] = json.charCodeAt(i);
          wsRef.current.send(buf);
        },
        onData: (data: string) => {
          if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN)
            return;

          const buf = new Uint8Array(data.length + 1);
          buf[0] = 0x00; // data prefix
          for (let i = 0; i < data.length; i++) buf[i + 1] = data.charCodeAt(i);
          wsRef.current.send(buf);
        },
      },
      addons: [fitRef.current],
    }),
    [theme]
  );

  const { instance: term, ref: termRef } = useXTerm(params);

  useEffect(() => {
    if (!term) return;

    term.clear();

    const ws = komodo_client().connect_terminal({
      query: {
        server,
        terminal,
        shell: "bash",
        // command: "clear",
      },
      on_login: () => {
        // console.log("logged in terminal");
      },
      on_open: resize,
      on_message: (e) => {
        term.write(new Uint8Array(e.data as ArrayBuffer));
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
  }, [term]);

  return (
    <div
      ref={termRef}
      className={cn("w-full h-[60vh]", selected ? "" : "hidden")}
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

const next_terminal_name = (terminals: string[]) => {
  for (let i = 1; i <= terminals.length + 1; i++) {
    const name = `Terminal ${i}`;
    if (!terminals.includes(name)) {
      return name;
    }
  }
  // This shouldn't happen
  return `Terminal -1`;
};
