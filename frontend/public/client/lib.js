import { UpdateStatus, } from "./types.js";
export * as Types from "./types.js";
export class CancelToken {
    cancelled;
    constructor() {
        this.cancelled = false;
    }
    cancel() {
        this.cancelled = true;
    }
}
/** Initialize a new client for Komodo */
export function KomodoClient(url, options) {
    const state = {
        jwt: options.type === "jwt" ? options.params.jwt : undefined,
        key: options.type === "api-key" ? options.params.key : undefined,
        secret: options.type === "api-key" ? options.params.secret : undefined,
    };
    const request = async (path, request) => new Promise(async (res, rej) => {
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
                const body = await response.json();
                res(body);
            }
            else {
                try {
                    const result = await response.json();
                    rej({ status: response.status, result });
                }
                catch (error) {
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
        }
        catch (error) {
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
    const auth = async (type, params) => await request("/auth", {
        type,
        params,
    });
    const user = async (type, params) => await request("/user", { type, params });
    const read = async (type, params) => await request("/read", { type, params });
    const write = async (type, params) => await request("/write", { type, params });
    const execute = async (type, params) => await request("/execute", { type, params });
    const execute_and_poll = async (type, params) => {
        const res = await execute(type, params);
        // Check if its a batch of updates or a single update;
        if (Array.isArray(res)) {
            const batch = res;
            return await Promise.all(batch.map(async (item) => {
                if (item.status === "Err") {
                    return item;
                }
                return await poll_update_until_complete(item.data._id?.$oid);
            }));
        }
        else {
            // it is a single update
            const update = res;
            return await poll_update_until_complete(update._id?.$oid);
        }
    };
    const poll_update_until_complete = async (update_id) => {
        while (true) {
            await new Promise((resolve) => setTimeout(resolve, 1000));
            const update = await read("GetUpdate", { id: update_id });
            if (update.status === UpdateStatus.Complete) {
                return update;
            }
        }
    };
    const core_version = () => read("GetVersion", {}).then((res) => res.version);
    const subscribe_to_update_websocket = async ({ on_update, on_login, on_close, retry_timeout_ms = 5_000, cancel = new CancelToken(), on_cancel, }) => {
        while (true) {
            if (cancel.cancelled) {
                on_cancel?.();
                return;
            }
            try {
                const ws = new WebSocket(url.replace("http", "ws") + "/ws/update");
                // Handle login on websocket open
                ws.addEventListener("open", () => {
                    const login_msg = options.type === "jwt"
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
                ws.addEventListener("message", ({ data }) => {
                    if (data == "LOGGED_IN")
                        return on_login?.();
                    on_update(JSON.parse(data));
                });
                if (on_close) {
                    ws.addEventListener("close", on_close);
                }
                // This while loop will end when the socket is closed
                while (ws.readyState !== WebSocket.CLOSING &&
                    ws.readyState !== WebSocket.CLOSED) {
                    if (cancel.cancelled)
                        ws.close();
                    // Sleep for a bit before checking for websocket closed
                    await new Promise((resolve) => setTimeout(resolve, 500));
                }
                // Sleep for a bit before retrying connection to avoid spam.
                await new Promise((resolve) => setTimeout(resolve, retry_timeout_ms));
            }
            catch (error) {
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
         * NOTE. These calls return immediately when the update is created, NOT when the execution task finishes.
         * To have the call only return when the task finishes, use [execute_and_poll_until_complete].
         *
         * https://docs.rs/komodo_client/latest/komodo_client/api/execute/index.html
         */
        execute,
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
        execute_and_poll,
        /**
         * Poll an Update (returned by the `execute` calls) until the `status` is `Complete`.
         * https://docs.rs/komodo_client/latest/komodo_client/entities/update/struct.Update.html#structfield.status.
         */
        poll_update_until_complete,
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
