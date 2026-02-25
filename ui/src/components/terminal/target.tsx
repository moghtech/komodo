import { komodo_client } from "@/lib/hooks";
import { TerminalCallbacks, Types } from "komodo_client";
import { useCallback } from "react";
import Terminal from ".";

export default function TargetTerminal({
  terminal,
  target,
  selected,
  _reconnect,
}: {
  terminal: string;
  target: Types.TerminalTarget;
  selected: boolean;
  _reconnect: boolean;
}) {
  const makeWs = useCallback(
    (callbacks: TerminalCallbacks) => {
      return komodo_client().connect_terminal({
        query: { target, terminal },
        ...callbacks,
      });
    },
    [target, terminal],
  );
  return (
    <Terminal makeWs={makeWs} selected={selected} _reconnect={_reconnect} />
  );
}
