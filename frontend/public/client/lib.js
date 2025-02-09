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
    const core_version = () => read("GetVersion", {}).then((res) => res.version);
    const subscribe_to_update_websocket = async ({ on_update, on_login, on_close, retry_timeout_ms = 3000, cancel = new CancelToken(), on_cancel, }) => {
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
