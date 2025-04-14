import {
  AuthResponses,
  ExecuteResponses,
  ReadResponses,
  UserResponses,
  WriteResponses,
} from "./responses.js";
import {
  AuthRequest,
  ExecuteRequest,
  ReadRequest,
  UpdateListItem,
  UserRequest,
  WriteRequest,
  WsLoginMessage,
} from "./types.js";

export * as Types from "./types.js";

type InitOptions =
  | { type: "jwt"; params: { jwt: string } }
  | { type: "api-key"; params: { key: string; secret: string } };

export class CancelToken {
  cancelled: boolean;
  constructor() {
    this.cancelled = false;
  }
  cancel() {
    this.cancelled = true;
  }
}

/** Initialize a new client for Komodo */
export function KomodoClient(url: string, options: InitOptions) {
  const state = {
    jwt: options.type === "jwt" ? options.params.jwt : undefined,
    key: options.type === "api-key" ? options.params.key : undefined,
    secret: options.type === "api-key" ? options.params.secret : undefined,
  };

  const request = async <Req, Res>(
    path: "/auth" | "/user" | "/read" | "/execute" | "/write",
    request: Req
  ): Promise<Res> =>
    new Promise(async (res, rej) => {
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
          const body: Res = await response.json();
          res(body);
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

  const auth = async <
    T extends AuthRequest["type"],
    Req extends Extract<AuthRequest, { type: T }>
  >(
    type: T,
    params: Req["params"]
  ) =>
    await request<
      { type: T; params: Req["params"] },
      AuthResponses[Req["type"]]
    >("/auth", {
      type,
      params,
    });

  const user = async <
    T extends UserRequest["type"],
    Req extends Extract<UserRequest, { type: T }>
  >(
    type: T,
    params: Req["params"]
  ) =>
    await request<
      { type: T; params: Req["params"] },
      UserResponses[Req["type"]]
    >("/user", { type, params });

  const read = async <
    T extends ReadRequest["type"],
    Req extends Extract<ReadRequest, { type: T }>
  >(
    type: T,
    params: Req["params"]
  ) =>
    await request<
      { type: T; params: Req["params"] },
      ReadResponses[Req["type"]]
    >("/read", { type, params });

  const write = async <
    T extends WriteRequest["type"],
    Req extends Extract<WriteRequest, { type: T }>
  >(
    type: T,
    params: Req["params"]
  ) =>
    await request<
      { type: T; params: Req["params"] },
      WriteResponses[Req["type"]]
    >("/write", { type, params });

  const execute = async <
    T extends ExecuteRequest["type"],
    Req extends Extract<ExecuteRequest, { type: T }>
  >(
    type: T,
    params: Req["params"]
  ) =>
    await request<
      { type: T; params: Req["params"] },
      ExecuteResponses[Req["type"]]
    >("/execute", { type, params });

  const core_version = () => read("GetVersion", {}).then((res) => res.version);

  const subscribe_to_update_websocket = async ({
    on_update,
    on_login,
    on_close,
    retry_timeout_ms = 5_000,
    cancel = new CancelToken(),
    on_cancel,
  }: {
    on_update: (update: UpdateListItem) => void;
    on_login?: () => void;
    on_open?: () => void;
    on_close?: () => void;
    retry_timeout_ms?: number;
    cancel?: CancelToken;
    on_cancel?: () => void;
  }) => {
    while (true) {
      if (cancel.cancelled) {
        on_cancel?.();
        return;
      }

      try {
        const ws = new WebSocket(url.replace("http", "ws") + "/ws/update");

        // Handle login on websocket open
        ws.addEventListener("open", () => {
          const login_msg: WsLoginMessage =
            options.type === "jwt"
              ? {
                  type: "Jwt",
                  params: {
                    jwt: options.params.jwt,
                  },
                }
              : {
                  type: "ApiKeys",
                  params: {
                    key: options.params.key,
                    secret: options.params.secret,
                  },
                };
          ws.send(JSON.stringify(login_msg));
        });

        ws.addEventListener("message", ({ data }: MessageEvent) => {
          if (data == "LOGGED_IN") return on_login?.();
          on_update(JSON.parse(data));
        });

        if (on_close) {
          ws.addEventListener("close", on_close);
        }

        // This while loop will end when the socket is closed
        while (
          ws.readyState !== WebSocket.CLOSING &&
          ws.readyState !== WebSocket.CLOSED
        ) {
          if (cancel.cancelled) ws.close();
          // Sleep for a bit before checking for websocket closed
          await new Promise((resolve) => setTimeout(resolve, 500));
        }

        // Sleep for a bit before retrying connection to avoid spam.
        await new Promise((resolve) => setTimeout(resolve, retry_timeout_ms));
      } catch (error) {
        console.error(error);
        // Sleep for a bit before retrying, maybe Komodo Core is down temporarily.
        await new Promise((resolve) => setTimeout(resolve, retry_timeout_ms));
      }
    }
  };

  return {
    /**
     * Call the `/auth` api.
     *
     * ```
     * const login_options = await komodo.auth("GetLoginOptions", {});
     * ```
     *
     * https://docs.rs/komodo_client/latest/komodo_client/api/auth/index.html
     */
    auth,
    /**
     * Call the `/user` api.
     *
     * ```
     * const { key, secret } = await komodo.user("CreateApiKey", {
     *   name: "my-api-key"
     * });
     * ```
     *
     * https://docs.rs/komodo_client/latest/komodo_client/api/user/index.html
     */
    user,
    /**
     * Call the `/read` api.
     *
     * ```
     * const stack = await komodo.read("GetStack", {
     *   stack: "my-stack"
     * });
     * ```
     *
     * https://docs.rs/komodo_client/latest/komodo_client/api/read/index.html
     */
    read,
    /**
     * Call the `/write` api.
     *
     * ```
     * const build = await komodo.write("UpdateBuild", {
     *   id: "my-build",
     *   config: {
     *     version: "1.0.4"
     *   }
     * });
     * ```
     *
     * https://docs.rs/komodo_client/latest/komodo_client/api/write/index.html
     */
    write,
    /**
     * Call the `/execute` api.
     *
     * ```
     * const update = await komodo.execute("DeployStack", {
     *   stack: "my-stack"
     * });
     * ```
     *
     * https://docs.rs/komodo_client/latest/komodo_client/api/execute/index.html
     */
    execute,
    /** Returns the version of Komodo Core the client is calling to. */
    core_version,
    /**
     * Subscribes to the update websocket with automatic reconnect loop.
     *
     * Note. Awaiting this method will never finish.
     */
    subscribe_to_update_websocket,
  };
}
