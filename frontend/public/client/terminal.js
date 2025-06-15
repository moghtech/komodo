export const terminal_methods = (url, state) => {
    const connect_terminal = ({ query, on_message, on_login, on_open, on_close, }) => {
        const url_query = new URLSearchParams(query).toString();
        const ws = new WebSocket(url.replace("http", "ws") + "/ws/terminal?" + url_query);
        // Handle login on websocket open
        ws.onopen = () => {
            const login_msg = state.jwt
                ? {
                    type: "Jwt",
                    params: {
                        jwt: state.jwt,
                    },
                }
                : {
                    type: "ApiKeys",
                    params: {
                        key: state.key,
                        secret: state.secret,
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
            }
            else {
                on_message?.(e);
            }
        };
        ws.onclose = () => on_close?.();
        return ws;
    };
    const execute_terminal = async (request, callbacks) => {
        const stream = await execute_terminal_stream(request);
        for await (const line of stream) {
            if (line.startsWith("__KOMODO_EXIT_CODE")) {
                await callbacks?.onFinish?.(line.split(":")[1]);
                return;
            }
            else {
                await callbacks?.onLine?.(line);
            }
        }
        // This is hit if no __KOMODO_EXIT_CODE is sent, ie early exit
        await callbacks?.onFinish?.("Early exit without code");
    };
    const execute_terminal_stream = (request) => execute_stream("/terminal/execute", request);
    const connect_container_exec = ({ query: { type, query }, on_message, on_login, on_open, on_close, }) => {
        const url_query = new URLSearchParams(query).toString();
        const ws = new WebSocket(url.replace("http", "ws") + `/ws/${type}/terminal?` + url_query);
        // Handle login on websocket open
        ws.onopen = () => {
            const login_msg = state.jwt
                ? {
                    type: "Jwt",
                    params: {
                        jwt: state.jwt,
                    },
                }
                : {
                    type: "ApiKeys",
                    params: {
                        key: state.key,
                        secret: state.secret,
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
            }
            else {
                on_message?.(e);
            }
        };
        ws.onclose = () => on_close?.();
        return ws;
    };
    const execute_container_exec = async (request, callbacks) => {
        const stream = await execute_container_exec_stream(request);
        for await (const line of stream) {
            if (line.startsWith("__KOMODO_EXIT_CODE")) {
                await callbacks?.onFinish?.(line.split(":")[1]);
                return;
            }
            else {
                await callbacks?.onLine?.(line);
            }
        }
        // This is hit if no __KOMODO_EXIT_CODE is sent, ie early exit
        await callbacks?.onFinish?.("Early exit without code");
    };
    const execute_container_exec_stream = (request) => execute_stream(`/terminal/execute/${request.type}`, request.query);
    const execute_stream = (path, request) => new Promise(async (res, rej) => {
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
                        .pipeThrough(new TransformStream({
                        start(_controller) {
                            this.tail = "";
                        },
                        transform(chunk, controller) {
                            const data = this.tail + chunk; // prepend any carry‑over
                            const parts = data.split(/\r?\n/); // split on CRLF or LF
                            this.tail = parts.pop(); // last item may be incomplete
                            for (const line of parts)
                                controller.enqueue(line);
                        },
                        flush(controller) {
                            if (this.tail)
                                controller.enqueue(this.tail); // final unterminated line
                        },
                    }));
                    res(stream);
                }
                else {
                    rej({
                        status: response.status,
                        result: { error: "No response body", trace: [] },
                    });
                }
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
    return {
        connect_terminal,
        execute_terminal,
        execute_terminal_stream,
        connect_container_exec,
        execute_container_exec,
        execute_container_exec_stream,
    };
};
