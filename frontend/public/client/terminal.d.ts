import { ClientState, Types } from "./lib";
import { ConnectTerminalQuery, ExecuteTerminalBody, InitTerminal } from "./types";
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
export declare const terminal_methods: (url: string, state: ClientState) => {
    connect_terminal: ({ query: { target, terminal, init }, on_message, on_login, on_open, on_close, }: {
        query: ConnectTerminalQuery;
    } & TerminalCallbacks) => WebSocket;
    execute_terminal: (request: ExecuteTerminalBody, callbacks?: ExecuteCallbacks) => Promise<void>;
    execute_terminal_stream: (request: ExecuteTerminalBody) => Promise<AsyncIterable<string>>;
    execute_server_terminal: ({ server, terminal, command, init, }: {
        server: string;
        terminal?: string;
        command: string;
        init?: InitTerminal;
    }, callbacks?: ExecuteCallbacks) => Promise<void>;
    execute_container_terminal: ({ server, container, terminal, command, init, }: {
        server: string;
        container: string;
        terminal?: string;
        command: string;
        init?: InitTerminal;
    }, callbacks?: ExecuteCallbacks) => Promise<void>;
    execute_deployment_terminal: ({ deployment, terminal, command, init, }: {
        deployment: string;
        terminal?: string;
        command: string;
        init?: InitTerminal;
    }, callbacks?: ExecuteCallbacks) => Promise<void>;
    execute_stack_service_terminal: ({ stack, service, terminal, command, init, }: {
        stack: string;
        service: string;
        terminal?: string;
        command: string;
        init?: InitTerminal;
    }, callbacks?: ExecuteCallbacks) => Promise<void>;
    execute_container_exec: ({ server, container, shell, command, terminal, recreate, }: {
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
    }, callbacks?: ExecuteCallbacks) => Promise<void>;
    execute_deployment_exec: ({ deployment, shell, command, terminal, recreate, }: {
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
    }, callbacks?: ExecuteCallbacks) => Promise<void>;
    execute_stack_exec: ({ stack, service, shell, command, terminal, recreate, }: {
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
    }, callbacks?: ExecuteCallbacks) => Promise<void>;
};
