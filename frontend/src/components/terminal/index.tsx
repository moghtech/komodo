import { cn } from "@lib/utils";
import { useTheme } from "@ui/theme";
import { FitAddon } from "@xterm/addon-fit";
import { ITheme } from "@xterm/xterm";
import { TerminalCallbacks } from "komodo_client";
import { useEffect, useMemo, useRef } from "react";
import { useXTerm, UseXTermProps } from "react-xtermjs";

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

export const Terminal = ({
  make_ws,
  selected,
  _reconnect,
  _clear,
}: {
  make_ws: (callbacks: TerminalCallbacks) => WebSocket;
  selected: boolean;
  _reconnect: boolean;
  _clear?: boolean;
}) => {
  const { currentTheme } = useTheme();
  const theme = currentTheme === "dark" ? DARK_THEME : LIGHT_THEME;
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

    const callbacks: TerminalCallbacks = {
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

    const ws = make_ws(callbacks);

    wsRef.current = ws;

    return () => {
      ws.close();
      wsRef.current = null;
    };
  }, [term, viewport, make_ws, selected, _reconnect]);

  useEffect(() => term?.clear(), [_clear]);

  return (
    <div
      ref={termRef}
      className={cn("w-full h-[65vh]", selected ? "" : "hidden")}
    />
  );
};
