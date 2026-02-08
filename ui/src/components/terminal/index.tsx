import { useComputedColorScheme } from "@mantine/core";
import { useEffect, useMemo, useRef } from "react";
import { ITheme } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { useXTerm, UseXTermProps } from "react-xtermjs";
import { TerminalCallbacks } from "komodo_client";

const LIGHT_THEME: ITheme = {
  background: "#f5f5f5",
  foreground: "#24292e",
  cursor: "#24292e",
  selectionBackground: "#c8d9fa",
};

const DARK_THEME: ITheme = {
  background: "#15171b",
  foreground: "#f6f8fa",
  cursor: "#f6f8fa",
  selectionBackground: "#6e778a",
};

export default function Terminal({
  makeWs,
  selected,
  _reconnect,
  _clear,
}: {
  makeWs: (callbacks: TerminalCallbacks) => WebSocket | undefined;
  selected: boolean;
  _reconnect: boolean;
  _clear?: boolean;
}) {
  const currentTheme = useComputedColorScheme();
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
        for (let i = 0; i < json.length; i++) buf[i] = json.charCodeAt(i);
        buf[json.length] = 0xff; // resize postfix
        wsRef.current.send(buf);
      }
      term.focus();
    }
  };

  const onStdin = (data: string) => {
    // This is data user writes to stdin
    if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) return;

    const buf = new Uint8Array(data.length + 1);
    for (let i = 0; i < data.length; i++) buf[i] = data.charCodeAt(i);
    buf[data.length] = 0x01; // forward data postfix
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
    [theme],
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
          }, 500) as any;
        });
      },
      on_close: () => {
        term.writeln("\r\n\x1b[33m[connection closed]\x1b[0m");
      },
    };

    const ws = makeWs(callbacks);

    if (!ws) return;

    wsRef.current = ws;

    return () => {
      ws.close();
      wsRef.current = null;
    };
  }, [term, viewport, makeWs, selected, _reconnect]);

  useEffect(() => term?.clear(), [_clear]);

  return (
    <div
      ref={termRef}
      style={{ display: selected ? undefined : "none", height: "calc(100vh - 30rem)" }}
    />
  );
}