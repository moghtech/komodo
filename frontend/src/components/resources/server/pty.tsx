import { Section } from "@components/layouts";
import { ReactNode, useEffect, useMemo, useRef, useState } from "react";
import { FitAddon } from "@xterm/addon-fit";
import { useXTerm, UseXTermProps } from "react-xtermjs";
import { komodo_client } from "@lib/hooks";

export const ServerPty = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther?: ReactNode;
}) => {
  // const { data: ptys, refetch: refetchPtys } = useRead("ListPtys", {
  //   server: id,
  //   fresh: true,
  // });
  // const { mutateAsync: delete_pty } = useWrite("DeletePty");
  // const [_selected, setSelected] = useLocalStorage<{
  //   selected: string | undefined;
  // }>(`server-${id}-selected-pty-v1`, { selected: undefined });
  // const selected = _selected.selected ?? `Pty ${(ptys?.length ?? 0) + 1}`;
  const wsRef = useRef<WebSocket | null>(null);
  const fitRef = useRef<FitAddon>(new FitAddon());
  const [_reconnect, _setReconnect] = useState(false);
  const triggerReconnect = () => _setReconnect((r) => !r);

  const sendResize = ({ rows, cols }: { rows: number; cols: number }) => {
    if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) return;

    const json = JSON.stringify({ rows, cols });
    const buf = new Uint8Array(json.length + 1);
    buf[0] = 0xff; // resize prefix
    for (let i = 0; i < json.length; i++) buf[i + 1] = json.charCodeAt(i);
    wsRef.current.send(buf);
  };

  const params: UseXTermProps = useMemo(
    () => ({
      options: {
        convertEol: true,
        cursorBlink: true,
        fontFamily: "monospace",
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
    []
  );

  const { instance: term, ref: termRef } = useXTerm(params);

  useEffect(() => {
    if (!term) return;

    console.log("set websocket");

    const ws = komodo_client().connect_pty({
      query: {
        server: id,
        pty: "Test Pty",
        shell: "bash",
      },
      on_open: () => {
        fitRef.current.fit();
        sendResize({ rows: term.rows, cols: term.cols });
        term.focus();
      },
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
  }, [term, _reconnect]);

  return (
    <Section titleOther={titleOther}>
      <div ref={termRef} className="w-full h-[50vh]" />
    </Section>
  );
};
