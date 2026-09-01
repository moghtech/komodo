import { TerminalCallbacks } from "komodo_client";
import { lazy, Suspense } from "react";

// The implementation pulls in @xterm / react-xtermjs,
// so it is loaded lazily to keep them out of the entry chunk.
const TerminalInner = lazy(() => import("./xterm"));

export default function Terminal(props: {
  makeWs: (callbacks: TerminalCallbacks) => WebSocket | undefined;
  selected: boolean;
  _reconnect: boolean;
  _clear?: boolean;
}) {
  return (
    <Suspense fallback={null}>
      <TerminalInner {...props} />
    </Suspense>
  );
}
