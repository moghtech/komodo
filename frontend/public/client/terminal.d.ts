import { ClientState } from "./lib";
import { ConnectContainerExecQuery, ConnectDeploymentExecQuery, ConnectStackExecQuery, ConnectTerminalQuery, ExecuteContainerExecBody, ExecuteDeploymentExecBody, ExecuteStackExecBody, ExecuteTerminalBody } from "./types";
export type TerminalCallbacks = {
    on_message?: (e: MessageEvent<any>) => void;
    on_login?: () => void;
    on_open?: () => void;
    on_close?: () => void;
};
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
    body: ExecuteContainerExecBody;
} | {
    type: "deployment";
    body: ExecuteDeploymentExecBody;
} | {
    type: "stack";
    body: ExecuteStackExecBody;
};
export type ExecuteCallbacks = {
    onLine?: (line: string) => void | Promise<void>;
    onFinish?: (code: string) => void | Promise<void>;
};
export declare const terminal_methods: (url: string, state: ClientState) => {
    connect_terminal: ({ query, on_message, on_login, on_open, on_close, }: {
        query: ConnectTerminalQuery;
    } & TerminalCallbacks) => WebSocket;
    execute_terminal: (request: ExecuteTerminalBody, callbacks?: ExecuteCallbacks) => Promise<void>;
    execute_terminal_stream: (request: ExecuteTerminalBody) => Promise<AsyncIterable<string>>;
    connect_exec: ({ query: { type, query }, on_message, on_login, on_open, on_close, }: {
        query: ConnectExecQuery;
    } & TerminalCallbacks) => WebSocket;
    connect_container_exec: ({ query, ...callbacks }: {
        query: ConnectContainerExecQuery;
    } & TerminalCallbacks) => WebSocket;
    execute_container_exec: (body: ExecuteContainerExecBody, callbacks?: ExecuteCallbacks) => Promise<void>;
    execute_container_exec_stream: (body: ExecuteContainerExecBody) => Promise<AsyncIterable<string>>;
    connect_deployment_exec: ({ query, ...callbacks }: {
        query: ConnectDeploymentExecQuery;
    } & TerminalCallbacks) => WebSocket;
    execute_deployment_exec: (body: ExecuteDeploymentExecBody, callbacks?: ExecuteCallbacks) => Promise<void>;
    execute_deployment_exec_stream: (body: ExecuteDeploymentExecBody) => Promise<AsyncIterable<string>>;
    connect_stack_exec: ({ query, ...callbacks }: {
        query: ConnectStackExecQuery;
    } & TerminalCallbacks) => WebSocket;
    execute_stack_exec: (body: ExecuteStackExecBody, callbacks?: ExecuteCallbacks) => Promise<void>;
    execute_stack_exec_stream: (body: ExecuteStackExecBody) => Promise<AsyncIterable<string>>;
};
