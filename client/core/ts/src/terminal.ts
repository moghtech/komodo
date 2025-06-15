import { ClientState, InitOptions } from "./lib";
import {
  ConnectContainerExecQuery,
  ConnectDeploymentExecQuery,
  ConnectStackExecQuery,
  ConnectTerminalQuery,
  ExecuteContainerExecBody,
  ExecuteDeploymentExecBody,
  ExecuteStackExecBody,
  ExecuteTerminalBody,
  WsLoginMessage,
} from "./types";

export type TerminalCallbacks = {
  on_message?: (e: MessageEvent<any>) => void;
  on_login?: () => void;
  on_open?: () => void;
  on_close?: () => void;
};

export type ConnectExecQuery =
  | {
      type: "container";
      query: ConnectContainerExecQuery;
    }
  | {
      type: "deployment";
      query: ConnectDeploymentExecQuery;
    }
  | {
      type: "stack";
      query: ConnectStackExecQuery;
    };

export type ExecuteExecBody =
  | {
      type: "container";
      body: ExecuteContainerExecBody;
    }
  | {
      type: "deployment";
      body: ExecuteDeploymentExecBody;
    }
  | {
      type: "stack";
      body: ExecuteStackExecBody;
    };

export type ExecuteCallbacks = {
  onLine?: (line: string) => void | Promise<void>;
  onFinish?: (code: string) => void | Promise<void>;
};

export const terminal_methods = (url: string, state: ClientState) => {
  const connect_terminal = ({
    query,
    on_message,
    on_login,
    on_open,
    on_close,
  }: {
    query: ConnectTerminalQuery;
  } & TerminalCallbacks) => {
    const url_query = new URLSearchParams(
      query as any as Record<string, string>
    ).toString();
    const ws = new WebSocket(
      url.replace("http", "ws") + "/ws/terminal?" + url_query
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
    callbacks?: ExecuteCallbacks
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

  const connect_container_exec = ({
    query,
    ...callbacks
  }: {
    query: ConnectContainerExecQuery;
  } & TerminalCallbacks) =>
    connect_exec({ query: { type: "container", query }, ...callbacks });

  const connect_deployment_exec = ({
    query,
    ...callbacks
  }: {
    query: ConnectDeploymentExecQuery;
  } & TerminalCallbacks) =>
    connect_exec({ query: { type: "deployment", query }, ...callbacks });

  const connect_stack_exec = ({
    query,
    ...callbacks
  }: {
    query: ConnectStackExecQuery;
  } & TerminalCallbacks) =>
    connect_exec({ query: { type: "stack", query }, ...callbacks });

  const connect_exec = ({
    query: { type, query },
    on_message,
    on_login,
    on_open,
    on_close,
  }: {
    query: ConnectExecQuery;
  } & TerminalCallbacks) => {
    const url_query = new URLSearchParams(
      query as any as Record<string, string>
    ).toString();
    const ws = new WebSocket(
      url.replace("http", "ws") + `/ws/${type}/terminal?` + url_query
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

  const execute_container_exec = (
    body: ExecuteContainerExecBody,
    callbacks?: ExecuteCallbacks
  ) => execute_exec({ type: "container", body }, callbacks);

  const execute_deployment_exec = (
    body: ExecuteDeploymentExecBody,
    callbacks?: ExecuteCallbacks
  ) => execute_exec({ type: "deployment", body }, callbacks);

  const execute_stack_exec = (
    body: ExecuteStackExecBody,
    callbacks?: ExecuteCallbacks
  ) => execute_exec({ type: "stack", body }, callbacks);

  const execute_exec = async (
    request: ExecuteExecBody,
    callbacks?: ExecuteCallbacks
  ) => {
    const stream = await execute_exec_stream(request);
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

  const execute_container_exec_stream = (body: ExecuteContainerExecBody) =>
    execute_exec_stream({ type: "container", body });

  const execute_deployment_exec_stream = (body: ExecuteDeploymentExecBody) =>
    execute_exec_stream({ type: "deployment", body });

  const execute_stack_exec_stream = (body: ExecuteStackExecBody) =>
    execute_exec_stream({ type: "stack", body });

  const execute_exec_stream = (request: ExecuteExecBody) =>
    execute_stream(`/terminal/execute/${request.type}`, request.body);

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
                } as Transformer<string, string> & { tail: string })
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

  return {
    connect_terminal,
    execute_terminal,
    execute_terminal_stream,
    connect_exec,
    connect_container_exec,
    execute_container_exec,
    execute_container_exec_stream,
    connect_deployment_exec,
    execute_deployment_exec,
    execute_deployment_exec_stream,
    connect_stack_exec,
    execute_stack_exec,
    execute_stack_exec_stream,
  };
};
