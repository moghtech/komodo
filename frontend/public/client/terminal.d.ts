import { ClientState } from "./lib";
import { ConnectContainerExecQuery, ConnectDeploymentExecQuery, ConnectStackExecQuery, ConnectTerminalQuery, ExecuteContainerExecBody, ExecuteDeploymentExecBody, ExecuteStackExecBody, ExecuteTerminalBody } from "./types";
export type ConnectExecQuery = {
    type: "container";
    query: ConnectContainerExecQuery;
} | {
    type: "deployment";
    query: ConnectDeploymentExecQuery;
} | {
    type: "stack";
    query: ConnectStackExecQuery;
};
export type ExecuteExecBody = {
    type: "container";
    query: ExecuteContainerExecBody;
} | {
    type: "deployment";
    query: ExecuteDeploymentExecBody;
} | {
    type: "stack";
    query: ExecuteStackExecBody;
};
export type TerminalCallbacks = {
    on_message?: (e: MessageEvent<any>) => void;
    on_login?: () => void;
    on_open?: () => void;
    on_close?: () => void;
};
export declare const terminal_methods: (url: string, state: ClientState) => {
    connect_terminal: ({ query, on_message, on_login, on_open, on_close, }: {
        query: ConnectTerminalQuery;
    } & TerminalCallbacks) => WebSocket;
    execute_terminal: (request: ExecuteTerminalBody, callbacks?: {
        onLine?: (line: string) => void | Promise<void>;
        onFinish?: (code: string) => void | Promise<void>;
    }) => Promise<void>;
    execute_terminal_stream: (request: ExecuteTerminalBody) => Promise<AsyncIterable<string>>;
    connect_container_exec: ({ query: { type, query }, on_message, on_login, on_open, on_close, }: {
        query: ConnectExecQuery;
    } & TerminalCallbacks) => WebSocket;
    execute_container_exec: (request: ExecuteExecBody, callbacks?: {
        onLine?: (line: string) => void | Promise<void>;
        onFinish?: (code: string) => void | Promise<void>;
    }) => Promise<void>;
    execute_container_exec_stream: (request: ExecuteExecBody) => Promise<AsyncIterable<string>>;
};
