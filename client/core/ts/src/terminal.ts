import { ClientState, Types } from "./lib";
import {
  ConnectTerminalQuery,
  ExecuteTerminalBody,
  InitTerminal,
  TerminalTarget,
  WsLoginMessage,
} from "./types";

export type TerminalCallbacks = {
  on_message?: (e: MessageEvent<any>) => void;
  on_login?: () => void;
  on_open?: () => void;
  on_close?: () => void;
};

export type ExecuteCallbacks = {
  onLine?: (line: string) => void | Promise<void>;
  onFinish?: (code: string) => void | Promise<void>;
};

export const terminal_methods = (url: string, state: ClientState) => {
  const connect_terminal = ({
    query: { target, terminal, init },
    on_message,
    on_login,
    on_open,
    on_close,
  }: {
    query: ConnectTerminalQuery;
  } & TerminalCallbacks) => {
    let url_query = connect_terminal_target_query(target);
    if (terminal) {
      url_query += `&terminal=${terminal}`;
    }
    if (init?.command) {
      url_query += `&init[command]=${init.command}`;
    }
    if (init?.recreate) {
      url_query += `&init[recreate]=${init.recreate}`;
    }
    if (init?.mode) {
      url_query += `&init[mode]=${init.mode}`;
    }
    const ws = new WebSocket(
      url.replace("http", "ws") + "/ws/terminal?" + url_query,
    );
    // Handle login on websocket open
    ws.onopen = () => {
      const login_msg: WsLoginMessage = state.jwt
        ? {
            type: "Jwt",
            params: {
              jwt: state.jwt,
            },
          }
        : {
            type: "ApiKeys",
            params: {
              key: state.key!,
              secret: state.secret!,
            },
          };
      ws.send(JSON.stringify(login_msg));
      on_open?.();
    };

    ws.onmessage = (e) => {
      if (e.data == "LOGGED_IN") {
        ws.binaryType = "arraybuffer";
        ws.onmessage = (e) => on_message?.(e);
        on_login?.();
        return;
      } else {
        on_message?.(e);
      }
    };

    ws.onclose = () => on_close?.();

    return ws;
  };

  const execute_terminal = async (
    request: ExecuteTerminalBody,
    callbacks?: ExecuteCallbacks,
  ) => {
    const stream = await execute_terminal_stream(request);
    for await (const line of stream) {
      if (line.startsWith("__KOMODO_EXIT_CODE")) {
        await callbacks?.onFinish?.(line.split(":")[1]);
        return;
      } else {
        await callbacks?.onLine?.(line);
      }
    }
    // This is hit if no __KOMODO_EXIT_CODE is sent, ie early exit
    await callbacks?.onFinish?.("Early exit without code");
  };

  const execute_terminal_stream = (request: ExecuteTerminalBody) =>
    execute_stream("/terminal/execute", request);

  const execute_stream = (path: string, request: any) =>
    new Promise<AsyncIterable<string>>(async (res, rej) => {
      try {
        let response = await fetch(url + path, {
          method: "POST",
          body: JSON.stringify(request),
          headers: {
            ...(state.jwt
              ? {
                  authorization: state.jwt,
                }
              : state.key && state.secret
                ? {
                    "x-api-key": state.key,
                    "x-api-secret": state.secret,
                  }
                : {}),
            "content-type": "application/json",
          },
        });
        if (response.status === 200) {
          if (response.body) {
            const stream = response.body
              .pipeThrough(new TextDecoderStream("utf-8"))
              .pipeThrough(
                new TransformStream<string, string>({
                  start(_controller) {
                    this.tail = "";
                  },
                  transform(chunk, controller) {
                    const data = this.tail + chunk; // prepend any carry‑over
                    const parts = data.split(/\r?\n/); // split on CRLF or LF
                    this.tail = parts.pop()!; // last item may be incomplete
                    for (const line of parts) controller.enqueue(line);
                  },
                  flush(controller) {
                    if (this.tail) controller.enqueue(this.tail); // final unterminated line
                  },
                } as Transformer<string, string> & { tail: string }),
              );
            res(stream);
          } else {
            rej({
              status: response.status,
              result: { error: "No response body", trace: [] },
            });
          }
        } else {
          try {
            const result = await response.json();
            rej({ status: response.status, result });
          } catch (error) {
            rej({
              status: response.status,
              result: {
                error: "Failed to get response body",
                trace: [JSON.stringify(error)],
              },
              error,
            });
          }
        }
      } catch (error) {
        rej({
          status: 1,
          result: {
            error: "Request failed with error",
            trace: [JSON.stringify(error)],
          },
          error,
        });
      }
    });

  const execute_server_terminal = (
    {
      server,
      terminal,
      command,
      init,
    }: {
      server: string;
      terminal?: string;
      command: string;
      init?: InitTerminal;
    },
    callbacks?: ExecuteCallbacks,
  ) =>
    execute_terminal(
      {
        target: { type: "Server", params: { server } },
        terminal,
        command,
        init,
      },
      callbacks,
    );

  const execute_container_terminal = async (
    {
      server,
      container,
      terminal,
      command,
      init,
    }: {
      server: string;
      container: string;
      terminal?: string;
      command: string;
      init?: InitTerminal;
    },
    callbacks?: ExecuteCallbacks,
  ) =>
    execute_terminal(
      {
        target: { type: "Container", params: { server, container } },
        terminal,
        command,
        init,
      },
      callbacks,
    );

  const execute_stack_service_terminal = async (
    {
      stack,
      service,
      terminal,
      command,
      init,
    }: {
      stack: string;
      service: string;
      terminal?: string;
      command: string;
      init?: InitTerminal;
    },
    callbacks?: ExecuteCallbacks,
  ) =>
    execute_terminal(
      {
        target: { type: "Stack", params: { stack, service } },
        terminal,
        command,
        init,
      },
      callbacks,
    );

  const execute_deployment_terminal = async (
    {
      deployment,
      terminal,
      command,
      init,
    }: {
      deployment: string;
      terminal?: string;
      command: string;
      init?: InitTerminal;
    },
    callbacks?: ExecuteCallbacks,
  ) =>
    execute_terminal(
      {
        target: { type: "Deployment", params: { deployment } },
        terminal,
        command,
        init,
      },
      callbacks,
    );

  // LEGACY METHODS
  const execute_container_exec = (
    {
      server,
      container,
      shell,
      command,
      terminal,
      recreate = Types.TerminalRecreateMode.DifferentCommand,
    }: {
      /** Server Id or name */
      server: string;
      /** The container name */
      container: string;
      /** The shell to use (eg. `sh` or `bash`) */
      shell: string;
      /** The command to execute. */
      command: string;
      /** The name of the terminal to connect to */
      terminal?: string;
      /** The behavior if  */
      recreate?: Types.TerminalRecreateMode;
    },
    callbacks?: ExecuteCallbacks,
  ) =>
    execute_container_terminal(
      {
        server,
        container,
        command,
        terminal,
        init: {
          command: shell,
          recreate,
        },
      },
      callbacks,
    );

  const execute_deployment_exec = (
    {
      deployment,
      shell,
      command,
      terminal,
      recreate = Types.TerminalRecreateMode.DifferentCommand,
    }: {
      /** Deployment Id or name */
      deployment: string;
      /** The shell to use (eg. `sh` or `bash`) */
      shell: string;
      /** The command to execute. */
      command: string;
      /** The name of the terminal to connect to */
      terminal?: string;
      /** The behavior if  */
      recreate?: Types.TerminalRecreateMode;
    },
    callbacks?: ExecuteCallbacks,
  ) =>
    execute_deployment_terminal(
      {
        deployment,
        command,
        terminal,
        init: {
          command: shell,
          recreate,
        },
      },
      callbacks,
    );

  const execute_stack_exec = (
    {
      stack,
      service,
      shell,
      command,
      terminal,
      recreate = Types.TerminalRecreateMode.DifferentCommand,
    }: {
      /** Stack Id or name */
      stack: string;
      /** The service name to connect to */
      service: string;
      /** The shell to use (eg. `sh` or `bash`) */
      shell: string;
      /** The command to execute. */
      command: string;
      /** The name of the terminal to connect to */
      terminal?: string;
      /** The behavior if  */
      recreate?: Types.TerminalRecreateMode;
    },
    callbacks?: ExecuteCallbacks,
  ) =>
    execute_stack_service_terminal(
      {
        stack,
        service,
        command,
        terminal,
        init: {
          command: shell,
          recreate,
        },
      },
      callbacks,
    );

  return {
    connect_terminal,
    execute_terminal,
    execute_terminal_stream,
    // Convenience methods
    execute_server_terminal,
    execute_container_terminal,
    execute_deployment_terminal,
    execute_stack_service_terminal,
    // Legacy convenience methods
    execute_container_exec,
    execute_deployment_exec,
    execute_stack_exec,
  };
};

const connect_terminal_target_query = (target: TerminalTarget) => {
  const base = `target[type]=${target.type}&`;
  switch (target.type) {
    case "Server":
      return base + `target[params][server]=${target.params.server}`;
    case "Container":
      return (
        base +
        `target[params][server]=${target.params.server}&target[params][container]=${target.params.container}`
      );
    case "Stack":
      return (
        base +
        `target[params][stack]=${target.params.stack}&target[params][service]=${target.params.service}`
      );
    case "Deployment":
      return base + `target[params][deployment]=${target.params.deployment}`;
  }
};
