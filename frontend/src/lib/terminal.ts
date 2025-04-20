import { RefObject } from "react";
import { komodo_client } from "./hooks";

export type TerminalCommand = {
  command: string;
  output: string;
  code?: string;
  next_cwd?: string;
};

export type TerminalHistory = {
  [terminal: string]: TerminalCommand[] | undefined;
};

export const KOMODO_EXIT_DATA = "__KOMODO_EXIT_DATA:";

export const terminal_history_manager = (
  server_id: string,
  logRef: RefObject<HTMLPreElement | null>,
  scrollRef: RefObject<HTMLDivElement | null>
) => {
  const history: TerminalHistory = {};

  const push_command = (terminal: string, command: string) => {
    if (!command) return;
    if (!history[terminal]) history[terminal] = [];
    history[terminal].push({ command, output: "" });
    scrollRef.current?.scroll({
      top: scrollRef.current.scrollHeight,
      behavior: "smooth",
    });
  };

  const push_line = (terminal: string, line: string) => {
    const data = history[terminal];
    if (!data || data.length === 0) return;
    const cmd = data[data.length - 1];
    if (line.startsWith(KOMODO_EXIT_DATA)) {
      const [_, code, next_cwd] = line.split(":");
      cmd.code = code;
      cmd.next_cwd = next_cwd;
      // if (logRef.current) {
      //   logRef.current.innerText = "";
      // }
    } else {
      if (cmd.output) {
        cmd.output += "\n";
      }
      cmd.output += line;
      if (logRef.current) {
        logRef.current.innerText = cmd.output;
      }
      scrollRef.current?.scroll({
        top: scrollRef.current.scrollHeight,
        behavior: "smooth",
      });
    }
  };

  const get_terminal_history = (
    terminal: string
  ): [
    string | undefined,
    TerminalCommand | undefined,
    TerminalCommand[] | undefined,
  ] => {
    const data = history[terminal];
    if (!data || data.length === 0) return [undefined, undefined, undefined];
    const last_cwd = data.findLast(
      (val) => val.next_cwd !== undefined
    )?.next_cwd;
    return [last_cwd, data[data.length - 1], data.slice(0, data.length - 1)];
  };

  const execute_terminal = async (
    terminal: string,
    command: string,
    onFinish?: () => void,
    onError?: (error: any) => void
  ) => {
    if (command.length === 0) return;
    push_command(terminal, command);
    try {
      const stream = await komodo_client().execute_terminal({
        server: server_id,
        terminal,
        command,
      });
      const reader = stream.getReader();
      if (logRef.current) {
        logRef.current.innerText = "";
      }
      try {
        while (true) {
          const { done, value: line } = await reader.read();
          if (line) push_line(terminal, line);
          if (done) break;
        }
      } finally {
        reader.releaseLock();
        onFinish?.();
      }
    } catch (error) {
      onError?.(error);
      // toast({
      //   title: "Execute terminal failed",
      //   description: JSON.stringify(error, undefined, 2),
      //   variant: "destructive",
      // });
    }
  };

  return {
    get_terminal_history,
    execute_terminal,
  };
};
