import { AuthResponses, ExecuteResponses, ReadResponses, UserResponses, WriteResponses } from "./responses.js";
import { AuthRequest, ConnectContainerExecQuery, ConnectDeploymentExecQuery, ConnectStackExecQuery, ConnectTerminalQuery, ExecuteRequest, ExecuteTerminalBody, ReadRequest, Update, UpdateListItem, UserRequest, WriteRequest } from "./types.js";
export * as Types from "./types.js";
export type InitOptions = {
    type: "jwt";
    params: {
        jwt: string;
    };
} | {
    type: "api-key";
    params: {
        key: string;
        secret: string;
    };
};
export declare class CancelToken {
    cancelled: boolean;
    constructor();
    cancel(): void;
}
export type ContainerExecQuery = {
    type: "container";
    query: ConnectContainerExecQuery;
} | {
    type: "deployment";
    query: ConnectDeploymentExecQuery;
} | {
    type: "stack";
    query: ConnectStackExecQuery;
};
export type TerminalCallbacks = {
    on_message?: (e: MessageEvent<any>) => void;
    on_login?: () => void;
    on_open?: () => void;
    on_close?: () => void;
};
/** Initialize a new client for Komodo */
export declare function KomodoClient(url: string, options: InitOptions): {
    /**
     * Call the `/auth` api.
     *
     * ```
     * const login_options = await komodo.auth("GetLoginOptions", {});
     * ```
     *
     * https://docs.rs/komodo_client/latest/komodo_client/api/auth/index.html
     */
    auth: <T extends AuthRequest["type"], Req extends Extract<AuthRequest, {
        type: T;
    }>>(type: T, params: Req["params"]) => Promise<AuthResponses[Req["type"]]>;
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
    user: <T extends UserRequest["type"], Req extends Extract<UserRequest, {
        type: T;
    }>>(type: T, params: Req["params"]) => Promise<UserResponses[Req["type"]]>;
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
    read: <T extends ReadRequest["type"], Req extends Extract<ReadRequest, {
        type: T;
    }>>(type: T, params: Req["params"]) => Promise<ReadResponses[Req["type"]]>;
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
    write: <T extends WriteRequest["type"], Req extends Extract<WriteRequest, {
        type: T;
    }>>(type: T, params: Req["params"]) => Promise<WriteResponses[Req["type"]]>;
    /**
     * Call the `/execute` api.
     *
     * ```
     * const update = await komodo.execute("DeployStack", {
     *   stack: "my-stack"
     * });
     * ```
     *
     * NOTE. These calls return immediately when the update is created, NOT when the execution task finishes.
     * To have the call only return when the task finishes, use [execute_and_poll_until_complete].
     *
     * https://docs.rs/komodo_client/latest/komodo_client/api/execute/index.html
     */
    execute: <T extends ExecuteRequest["type"], Req extends Extract<ExecuteRequest, {
        type: T;
    }>>(type: T, params: Req["params"]) => Promise<ExecuteResponses[Req["type"]]>;
    /**
     * Call the `/execute` api, and poll the update until the task has completed.
     *
     * ```
     * const update = await komodo.execute_and_poll("DeployStack", {
     *   stack: "my-stack"
     * });
     * ```
     *
     * https://docs.rs/komodo_client/latest/komodo_client/api/execute/index.html
     */
    execute_and_poll: <T extends ExecuteRequest["type"], Req extends Extract<ExecuteRequest, {
        type: T;
    }>>(type: T, params: Req["params"]) => Promise<Update | (Update | {
        status: "Err";
        data: import("./types.js").BatchExecutionResponseItemErr;
    })[]>;
    /**
     * Poll an Update (returned by the `execute` calls) until the `status` is `Complete`.
     * https://docs.rs/komodo_client/latest/komodo_client/entities/update/struct.Update.html#structfield.status.
     */
    poll_update_until_complete: (update_id: string) => Promise<Update>;
    /** Returns the version of Komodo Core the client is calling to. */
    core_version: () => Promise<string>;
    /**
     * Connects to update websocket, performs login and attaches handlers,
     * and returns the WebSocket handle.
     */
    get_update_websocket: ({ on_update, on_login, on_open, on_close, }: {
        on_update: (update: UpdateListItem) => void;
        on_login?: () => void;
        on_open?: () => void;
        on_close?: () => void;
    }) => WebSocket;
    /**
     * Subscribes to the update websocket with automatic reconnect loop.
     *
     * Note. Awaiting this method will never finish.
     */
    subscribe_to_update_websocket: ({ on_update, on_open, on_login, on_close, retry, retry_timeout_ms, cancel, on_cancel, }: {
        on_update: (update: UpdateListItem) => void;
        on_login?: () => void;
        on_open?: () => void;
        on_close?: () => void;
        retry?: boolean;
        retry_timeout_ms?: number;
        cancel?: CancelToken;
        on_cancel?: () => void;
    }) => Promise<void>;
    /**
     * Subscribes to terminal io over websocket message,
     * for use with xtermjs.
     */
    connect_terminal: ({ query, on_message, on_login, on_open, on_close, }: {
        query: ConnectTerminalQuery;
    } & TerminalCallbacks) => WebSocket;
    /**
     * Subscribes to container exec io over websocket message,
     * for use with xtermjs. Can connect to Deployment, Stack,
     * or any container on a Server. The permission used to allow the connection
     * depends on `query.type`.
     */
    connect_container_exec: ({ query: { type, query }, on_message, on_login, on_open, on_close, }: {
        query: ContainerExecQuery;
    } & TerminalCallbacks) => WebSocket;
    /**
     * Executes a command on a given Server / terminal,
     * and returns a stream to process the output as it comes in.
     *
     * Note. The final line of the stream will usually be
     * `__KOMODO_EXIT_CODE__:0`. The number
     * is the exit code of the command.
     *
     * If this line is NOT present, it means the stream
     * was terminated early, ie like running `exit`.
     *
     * ```ts
     * const stream = await komodo.execute_terminal_stream({
     *   server: "my-server",
     *   terminal: "name",
     *   command: 'for i in {1..3}; do echo "$i"; sleep 1; done',
     * });
     *
     * for await (const line of stream) {
     *   console.log(line);
     * }
     * ```
     */
    execute_terminal_stream: (request: ExecuteTerminalBody) => Promise<AsyncIterable<string>>;
    /**
     * Executes a command on a given Server / terminal,
     * and gives a callback to handle the output as it comes in.
     *
     * ```ts
     * const stream = await komodo.execute_terminal(
     *   {
     *     server: "my-server",
     *     terminal: "name",
     *     command: 'for i in {1..3}; do echo "$i"; sleep 1; done',
     *   },
     *   {
     *     onLine: (line) => console.log(line),
     *     onFinish: (code) => console.log("Finished:", code),
     *   }
     * );
     * ```
     */
    execute_terminal: (request: ExecuteTerminalBody, callbacks?: {
        onLine?: (line: string) => void | Promise<void>;
        onFinish?: (code: string) => void | Promise<void>;
    }) => Promise<void>;
};
