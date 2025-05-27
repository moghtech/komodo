import { komodo_client, useLocalStorage } from "@lib/hooks";
import { cn } from "@lib/utils";
import { useTheme } from "@ui/theme";
import { FitAddon } from "@xterm/addon-fit";
import { ITheme } from "@xterm/xterm";
import { ReactNode, useEffect, useMemo, useRef, useState } from "react";
import { useXTerm, UseXTermProps } from "react-xtermjs";
import { Section } from "./layouts";
import { CardTitle } from "@ui/card";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { Input } from "@ui/input";
import { Button } from "@ui/button";
import { RefreshCcw } from "lucide-react";

const BASE_SHELLS = ["sh", "bash"];

export const ContainerTerminal = ({
  server,
  container,
  titleOther,
}: {
  server: string;
  container: string;
  titleOther?: ReactNode;
}) => {
  const [_reconnect, _setReconnect] = useState(false);
  const triggerReconnect = () => _setReconnect((r) => !r);
  const [_clear, _setClear] = useState(false);
  const [shell, setShell] = useLocalStorage(
    `server-${server}-${container}-shell-v1`,
    "sh"
  );
  const [otherShell, setOtherShell] = useState("");

  return (
    <Section
      titleOther={titleOther}
      actions={
        <div className="flex items-center gap-4 mr-[16px]">
          <CardTitle className="text-muted-foreground flex items-center gap-2">
            docker exec -it {container}
            <Select value={shell} onValueChange={setShell}>
              <SelectTrigger className="w-[120px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectGroup>
                  {[
                    ...BASE_SHELLS,
                    ...(!BASE_SHELLS.includes(shell) ? [shell] : []),
                  ].map((shell) => (
                    <SelectItem key={shell} value={shell}>
                      {shell}
                    </SelectItem>
                  ))}
                  <Input
                    placeholder="other"
                    value={otherShell}
                    onChange={(e) => setOtherShell(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        setShell(otherShell);
                        setOtherShell("");
                      } else {
                        e.stopPropagation();
                      }
                    }}
                  />
                </SelectGroup>
              </SelectContent>
            </Select>
          </CardTitle>
          <Button
            className="flex items-center gap-2"
            variant="secondary"
            onClick={() => triggerReconnect()}
          >
            Reconnect
            <RefreshCcw className="w-4 h-4" />
          </Button>
        </div>
      }
    >
      <div className="min-h-[65vh]">
        <Terminal
          query={{ server, container, shell }}
          selected={true}
          _clear={_clear}
          _reconnect={_reconnect}
        />
      </div>
    </Section>
  );
};

export const Terminal = ({
  query: { server, ...query },
  selected,
  _clear,
  _reconnect,
}: {
  query: { server: string } & (
    | { terminal: string; container?: undefined; shell?: undefined }
    | { container: string; shell: string; terminal?: undefined }
  );
  selected: boolean;
  _clear?: boolean;
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

    const options = {
      on_login: () => {
        // console.log("logged in terminal");
      },
      on_open: resize,
      on_message: (e: MessageEvent<any>) => {
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
    };

    const ws = query.container
      ? komodo_client().connect_container_exec({
          query: {
            type: "container",
            query: {
              server,
              container: query.container,
              shell: query.shell,
            },
          },
          ...options,
        })
      : komodo_client().connect_terminal({
          query: {
            server,
            terminal: query.terminal!,
          },
          ...options,
        });

    wsRef.current = ws;

    return () => {
      ws.close();
      wsRef.current = null;
    };
  }, [term, viewport, selected, query.shell, _reconnect]);

  useEffect(() => term?.clear(), [_clear]);

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
