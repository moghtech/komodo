import { KOMODO_BASE_URL } from "@main";
import { KomodoClient, Types } from "komodo_client";
import {
  AuthResponses,
  ExecuteResponses,
  ReadResponses,
  UserResponses,
  WriteResponses,
} from "komodo_client/dist/responses";
import {
  UseMutationOptions,
  UseQueryOptions,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { UsableResource } from "@types";
import { useToast } from "@ui/use-toast";
import { atom, useAtom } from "jotai";
import { atomFamily } from "jotai/utils";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useParams } from "react-router-dom";
import {
  has_minimum_permissions,
  RESOURCE_TARGETS,
  resourceTargetFromTerminalTarget,
} from "./utils";

export const atomWithStorage = <T>(key: string, init: T) => {
  const stored = localStorage.getItem(key);
  const inner = atom(stored ? JSON.parse(stored) : init);

  return atom(
    (get) => get(inner),
    (_, set, newValue) => {
      set(inner, newValue);
      localStorage.setItem(key, JSON.stringify(newValue));
    }
  );
};

type LoginTokens = {
  /** Current User ID */
  current: string | undefined;
  /** Array of logged in user ids / tokens */
  tokens: Array<Types.JwtResponse>;
};

const LOGIN_TOKENS_KEY = "komodo-auth-tokens-v1";

export const LOGIN_TOKENS = (() => {
  const stored = localStorage.getItem(LOGIN_TOKENS_KEY);

  let tokens: LoginTokens = stored
    ? JSON.parse(stored)
    : { current: undefined, tokens: [] };

  const update_local_storage = () => {
    localStorage.setItem(LOGIN_TOKENS_KEY, JSON.stringify(tokens));
  };

  const accounts = () => {
    const current = tokens.tokens.find((t) => t.user_id === tokens.current);
    const filtered = tokens.tokens.filter((t) => t.user_id !== tokens.current);
    return current ? [current, ...filtered] : filtered;
  };

  const add_and_change = (user_id: string, jwt: string) => {
    const filtered = tokens.tokens.filter((t) => t.user_id !== user_id);
    filtered.push({ user_id, jwt });
    filtered.sort();
    tokens = {
      current: user_id,
      tokens: filtered,
    };
    update_local_storage();
  };

  const remove = (user_id: string) => {
    const filtered = tokens.tokens.filter((t) => t.user_id !== user_id);
    tokens = {
      current:
        tokens.current === user_id ? filtered[0]?.user_id : tokens.current,
      tokens: filtered,
    };
    update_local_storage();
  };

  const remove_all = () => {
    tokens = {
      current: undefined,
      tokens: [],
    };
    update_local_storage();
  };

  const change = (to_id: string) => {
    tokens = {
      current: to_id,
      tokens: tokens.tokens,
    };
    update_local_storage();
  };

  return {
    jwt: () =>
      tokens.current
        ? (tokens.tokens.find((t) => t.user_id === tokens.current)?.jwt ?? "")
        : "",
    accounts,
    add_and_change,
    remove,
    remove_all,
    change,
  };
})();

export const komodo_client = () =>
  KomodoClient(KOMODO_BASE_URL, {
    type: "jwt",
    params: { jwt: LOGIN_TOKENS.jwt() },
  });

// ============== RESOLVER ==============

export const useLoginOptions = () => {
  return useQuery({
    queryKey: ["GetLoginOptions"],
    queryFn: () => komodo_client().auth("GetLoginOptions", {}),
  });
};

export const useUser = () => {
  const userReset = useUserReset();
  const hasJwt = !!LOGIN_TOKENS.jwt();

  const query = useQuery({
    queryKey: ["GetUser"],
    queryFn: () => komodo_client().auth("GetUser", {}),
    refetchInterval: 30_000,
    enabled: hasJwt,
  });

  useEffect(() => {
    if (query.data && query.error) {
      userReset();
    }
  }, [query.data, query.error]);

  return query;
};

export const useUserInvalidate = () => {
  const qc = useQueryClient();
  return () => {
    qc.invalidateQueries({ queryKey: ["GetUser"] });
  };
};

export const useUserReset = () => {
  const qc = useQueryClient();
  return () => {
    qc.resetQueries({ queryKey: ["GetUser"] });
  };
};

export const useManageUser = <
  T extends Types.UserRequest["type"],
  R extends Extract<Types.UserRequest, { type: T }>,
  P extends R["params"],
  C extends Omit<
    UseMutationOptions<UserResponses[T], unknown, P, unknown>,
    "mutationKey" | "mutationFn"
  >,
>(
  type: T,
  config?: C
) => {
  const { toast } = useToast();
  return useMutation({
    mutationKey: [type],
    mutationFn: (params: P) => komodo_client().user<T, R>(type, params),
    onError: (e: { result: { error?: string; trace?: string[] } }, v, c) => {
      console.log("Auth error:", e);
      const msg = e.result?.error ?? "Unknown error. See console.";
      const detail = e.result?.trace
        ?.map((msg) => msg[0].toUpperCase() + msg.slice(1))
        .join(" | ");
      let msg_log = msg ? msg[0].toUpperCase() + msg.slice(1) + " | " : "";
      if (detail) {
        msg_log += detail + " | ";
      }
      toast({
        title: `Request ${type} Failed`,
        description: `${msg_log}See console for details`,
        variant: "destructive",
      });
      config?.onError && config.onError(e, v, c);
    },
    ...config,
  });
};

export const useAuth = <
  T extends Types.AuthRequest["type"],
  R extends Extract<Types.AuthRequest, { type: T }>,
  P extends R["params"],
  C extends Omit<
    UseMutationOptions<AuthResponses[T], unknown, P, unknown>,
    "mutationKey" | "mutationFn"
  >,
>(
  type: T,
  config?: C
) => {
  const { toast } = useToast();
  return useMutation({
    mutationKey: [type],
    mutationFn: (params: P) => komodo_client().auth<T, R>(type, params),
    onError: (e: { result: { error?: string; trace?: string[] } }, v, c) => {
      console.log("Auth error:", e);
      const msg = e.result.error ?? "Unknown error. See console.";
      const detail = e.result?.trace
        ?.map((msg) => msg[0].toUpperCase() + msg.slice(1))
        .join(" | ");
      let msg_log = msg ? msg[0].toUpperCase() + msg.slice(1) + " | " : "";
      if (detail) {
        msg_log += detail + " | ";
      }
      toast({
        title: `Auth request ${type} failed`,
        description: `${msg_log}See console for details`,
        variant: "destructive",
      });
      config?.onError && config.onError(e, v, c);
    },
    ...config,
  });
};

export const useRead = <
  T extends Types.ReadRequest["type"],
  R extends Extract<Types.ReadRequest, { type: T }>,
  P extends R["params"],
  C extends Omit<
    UseQueryOptions<
      ReadResponses[R["type"]],
      unknown,
      ReadResponses[R["type"]],
      (T | P)[]
    >,
    "queryFn" | "queryKey"
  >,
>(
  type: T,
  params: P,
  config?: C
) => {
  const hasJwt = !!LOGIN_TOKENS.jwt();
  return useQuery({
    queryKey: [type, params],
    queryFn: () => komodo_client().read<T, R>(type, params),
    enabled: hasJwt && config?.enabled !== false,
    ...config,
  });
};

export const useInvalidate = () => {
  const qc = useQueryClient();
  return <
    Type extends Types.ReadRequest["type"],
    Params extends Extract<Types.ReadRequest, { type: Type }>["params"],
  >(
    ...keys: Array<[Type] | [Type, Params]>
  ) => keys.forEach((key) => qc.invalidateQueries({ queryKey: key }));
};

export const useWrite = <
  T extends Types.WriteRequest["type"],
  R extends Extract<Types.WriteRequest, { type: T }>,
  P extends R["params"],
  C extends Omit<
    UseMutationOptions<WriteResponses[R["type"]], unknown, P, unknown>,
    "mutationKey" | "mutationFn"
  >,
>(
  type: T,
  config?: C
) => {
  const { toast } = useToast();
  return useMutation({
    mutationKey: [type],
    mutationFn: (params: P) => komodo_client().write<T, R>(type, params),
    onError: (e: { result: { error?: string; trace?: string[] } }, v, c) => {
      console.log("Write error:", e);
      const msg = e.result.error ?? "Unknown error. See console.";
      const detail = e.result?.trace
        ?.map((msg) => msg[0].toUpperCase() + msg.slice(1))
        .join(" | ");
      let msg_log = msg ? msg[0].toUpperCase() + msg.slice(1) + " | " : "";
      if (detail) {
        msg_log += detail + " | ";
      }
      toast({
        title: `Write request ${type} failed`,
        description: `${msg_log}See console for details`,
        variant: "destructive",
      });
      config?.onError && config.onError(e, v, c);
    },
    ...config,
  });
};

export const useExecute = <
  T extends Types.ExecuteRequest["type"],
  R extends Extract<Types.ExecuteRequest, { type: T }>,
  P extends R["params"],
  C extends Omit<
    UseMutationOptions<ExecuteResponses[T], unknown, P, unknown>,
    "mutationKey" | "mutationFn"
  >,
>(
  type: T,
  config?: C
) => {
  const { toast } = useToast();
  return useMutation({
    mutationKey: [type],
    mutationFn: (params: P) => komodo_client().execute<T, R>(type, params),
    onError: (e: { result: { error?: string; trace?: string[] } }, v, c) => {
      console.log("Execute error:", e);
      const msg = e.result.error ?? "Unknown error. See console.";
      const detail = e.result?.trace
        ?.map((msg) => msg[0].toUpperCase() + msg.slice(1))
        .join(" | ");
      let msg_log = msg ? msg[0].toUpperCase() + msg.slice(1) + " | " : "";
      if (detail) {
        msg_log += detail + " | ";
      }
      toast({
        title: `Execute request ${type} failed`,
        description: `${msg_log}See console for details`,
        variant: "destructive",
      });
      config?.onError && config.onError(e, v, c);
    },
    ...config,
  });
};

// ============== UTILITY ==============

export const useResourceName = (type: UsableResource) => {
  const resources = useRead(`List${type}s`, {}).data;
  return useCallback(
    (id: string) => resources?.find((resource) => resource.id === id)?.name,
    [resources]
  );
};

export const useResourceParamType = () => {
  const type = useParams().type;
  if (!type) return undefined;
  if (type === "resource-syncs") return "ResourceSync";
  return (type[0].toUpperCase() + type.slice(1, -1)) as UsableResource;
};

type ResourceMap = {
  [Resource in UsableResource]: Types.ResourceListItem<unknown>[] | undefined;
};

export const useAllResources = (): ResourceMap => {
  return {
    Swarm: useRead("ListSwarms", {}).data,
    Server: useRead("ListServers", {}).data,
    Stack: useRead("ListStacks", {}).data,
    Deployment: useRead("ListDeployments", {}).data,
    Build: useRead("ListBuilds", {}).data,
    Repo: useRead("ListRepos", {}).data,
    Procedure: useRead("ListProcedures", {}).data,
    Action: useRead("ListActions", {}).data,
    Builder: useRead("ListBuilders", {}).data,
    Alerter: useRead("ListAlerters", {}).data,
    ResourceSync: useRead("ListResourceSyncs", {}).data,
  };
};

// Returns true if Komodo has no resources.
export const useNoResources = () => {
  const resources = useAllResources();
  for (const target of RESOURCE_TARGETS) {
    if (resources[target] && resources[target].length) {
      return false;
    }
  }
  return true;
};

/** returns function that takes a resource target and checks if it exists */
export const useCheckResourceExists = () => {
  const resources = useAllResources();
  return (target: Types.ResourceTarget) => {
    return (
      resources[target.type as UsableResource]?.some(
        (resource) => resource.id === target.id
      ) || false
    );
  };
};

export const useFilterResources = <Info>(
  resources?: Types.ResourceListItem<Info>[],
  search?: string
) => {
  const tags = useTagsFilter();
  const searchSplit = search?.toLowerCase()?.split(" ") || [];
  return (
    resources?.filter(
      (resource) =>
        tags.every((tag: string) => resource.tags.includes(tag)) &&
        (searchSplit.length > 0
          ? searchSplit.every((search) =>
              resource.name.toLowerCase().includes(search)
            )
          : true)
    ) ?? []
  );
};

export const usePushRecentlyViewed = ({ type, id }: Types.ResourceTarget) => {
  const userInvalidate = useUserInvalidate();

  const push = useManageUser("PushRecentlyViewed", {
    onSuccess: userInvalidate,
  }).mutate;

  const exists = useRead(`List${type as UsableResource}s`, {}).data?.find(
    (r) => r.id === id
  )
    ? true
    : false;

  useEffect(() => {
    exists && push({ resource: { type, id } });
  }, [exists, push]);

  return () => push({ resource: { type, id } });
};

export const useSetTitle = (more?: string) => {
  const info = useRead("GetCoreInfo", {}).data;
  const title = more ? `${more} | ${info?.title}` : info?.title;
  useEffect(() => {
    if (title) {
      document.title = title;
    }
  }, [title]);
};

const tagsAtom = atomWithStorage<string[]>("tags-v0", []);

export const useTags = () => {
  const [tags, setTags] = useAtom<string[]>(tagsAtom);

  const add_tag = (tag_id: string) => setTags([...tags, tag_id]);
  const remove_tag = (tag_id: string) =>
    setTags(tags.filter((id) => id !== tag_id));
  const toggle_tag = (tag_id: string) => {
    if (tags.includes(tag_id)) {
      remove_tag(tag_id);
    } else {
      add_tag(tag_id);
    }
  };
  const clear_tags = () => setTags([]);

  return {
    tags,
    add_tag,
    remove_tag,
    toggle_tag,
    clear_tags,
  };
};

export const useTagsFilter = () => {
  const [tags] = useAtom<string[]>(tagsAtom);
  return tags;
};

export type LocalStorageSetter<T> = (state: T) => T;

export const useLocalStorage = <T>(
  key: string,
  init: T
): [T, (state: T | LocalStorageSetter<T>) => void] => {
  const stored = localStorage.getItem(key);
  const parsed = stored ? (JSON.parse(stored) as T) : undefined;
  const [state, inner_set] = useState<T>(parsed ?? init);
  const set = (state: T | LocalStorageSetter<T>) => {
    inner_set((prev_state) => {
      const new_val =
        typeof state === "function"
          ? (state as LocalStorageSetter<T>)(prev_state)
          : state;
      localStorage.setItem(key, JSON.stringify(new_val));
      return new_val;
    });
  };
  return [state, set];
};

export const useKeyListener = (listenKey: string, onPress: () => void) => {
  useEffect(() => {
    const keydown = (e: KeyboardEvent) => {
      // This will ignore Shift + listenKey if it is sent from input / textarea
      const target = e.target as HTMLElement | null;
      if (target?.matches("input") || target?.matches("textarea")) return;
      if (e.key === listenKey) {
        e.preventDefault();
        onPress();
      }
    };
    document.addEventListener("keydown", keydown);
    return () => document.removeEventListener("keydown", keydown);
  });
};

export const useShiftKeyListener = (listenKey: string, onPress: () => void) => {
  useEffect(() => {
    const keydown = (e: KeyboardEvent) => {
      // This will ignore Shift + listenKey if it is sent from input / textarea
      const target = e.target as HTMLElement | null;
      if (target?.matches("input") || target?.matches("textarea")) return;
      if (e.shiftKey && e.key === listenKey) {
        e.preventDefault();
        onPress();
      }
    };
    document.addEventListener("keydown", keydown);
    return () => document.removeEventListener("keydown", keydown);
  });
};

/** Listens for ctrl (or CMD on mac) + the listenKey */
export const useCtrlKeyListener = (listenKey: string, onPress: () => void) => {
  useEffect(() => {
    const keydown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === listenKey) {
        e.preventDefault();
        onPress();
      }
    };
    document.addEventListener("keydown", keydown);
    return () => document.removeEventListener("keydown", keydown);
  });
};

export interface PromptHotkeysConfig {
  /** Function to call when Enter is pressed (confirm action) */
  onConfirm?: () => void;
  /** Function to call when Escape is pressed (cancel/close action) */
  onCancel?: () => void;
  /** Whether the hotkeys are enabled. Defaults to true */
  enabled?: boolean;
  /** Whether to ignore hotkeys when inside input/textarea elements. Defaults to true */
  ignoreInputs?: boolean;
  /** Whether the confirm action is disabled (e.g., form validation failed) */
  confirmDisabled?: boolean;
}

/**
 * Hook that provides standard prompt/dialog hotkey behavior:
 * - Enter: Confirm/submit action
 * - Escape: Cancel/close action
 */
export const usePromptHotkeys = ({
  enabled = true,
  onConfirm,
  onCancel,
  ignoreInputs = true,
  confirmDisabled = false,
}: PromptHotkeysConfig) => {
  useEffect(() => {
    if (!enabled) return;

    const findConfirmButton = (): HTMLButtonElement | null => {
      const dialogContainers = document.querySelectorAll(
        '[role="dialog"], [data-state="open"], .dialog-content'
      );
      for (const container of dialogContainers) {
        const button = container.querySelector(
          "[data-confirm-button]:not([disabled])"
        ) as HTMLButtonElement;
        if (button) return button;
      }

      return document.querySelector(
        "[data-confirm-button]:not([disabled])"
      ) as HTMLButtonElement;
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      if (ignoreInputs) {
        const target = e.target as HTMLElement;
        if (
          target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.tagName === "SELECT" ||
          target.isContentEditable
        ) {
          return;
        }
      }

      switch (e.key) {
        case "Enter":
          if (onConfirm && !confirmDisabled) {
            e.preventDefault();
            const confirmButton = findConfirmButton();
            if (confirmButton) {
              confirmButton.click();
            } else {
              onConfirm();
            }
          }
          break;
        case "Escape":
          if (onCancel) {
            e.preventDefault();
            onCancel();
          }
          break;
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [enabled, onConfirm, onCancel, ignoreInputs, confirmDisabled]);
};

export type WebhookIntegration = "Github" | "Gitlab";
export type WebhookIntegrations = {
  [key: string]: WebhookIntegration;
};

const WEBHOOK_INTEGRATIONS_ATOM = atomWithStorage<WebhookIntegrations>(
  "webhook-integrations-v2",
  {}
);

export const useWebhookIntegrations = () => {
  const [integrations, setIntegrations] = useAtom<WebhookIntegrations>(
    WEBHOOK_INTEGRATIONS_ATOM
  );
  return {
    integrations,
    setIntegration: (provider: string, integration: WebhookIntegration) =>
      setIntegrations({
        ...integrations,
        [provider]: integration,
      }),
  };
};

export const getWebhookIntegration = (
  integrations: WebhookIntegrations,
  git_provider: string
) => {
  return integrations[git_provider]
    ? integrations[git_provider]
    : git_provider.includes("gitlab")
      ? "Gitlab"
      : "Github";
};

export type WebhookIdOrName = "Id" | "Name";

const WEBHOOK_ID_OR_NAME_ATOM = atomWithStorage<WebhookIdOrName>(
  "webhook-id-or-name-v1",
  "Id"
);

export const useWebhookIdOrName = () => {
  return useAtom<WebhookIdOrName>(WEBHOOK_ID_OR_NAME_ATOM);
};

export type Dimensions = { width: number; height: number };
export const useWindowDimensions = () => {
  const [dimensions, setDimensions] = useState<Dimensions>({
    width: 0,
    height: 0,
  });
  useEffect(() => {
    const callback = () => {
      setDimensions({
        width: window.screen.availWidth,
        height: window.screen.availHeight,
      });
    };
    callback();
    window.addEventListener("resize", callback);
    return () => {
      window.removeEventListener("resize", callback);
    };
  }, []);
  return dimensions;
};

const selected_resources = atomFamily((_: UsableResource) =>
  atom<string[]>([])
);
export const useSelectedResources = (type: UsableResource) =>
  useAtom(selected_resources(type));

const filter_by_update_available = atomWithStorage<boolean>(
  "update-available-filter-v1",
  false
);
export const useFilterByUpdateAvailable: () => [boolean, () => void] = () => {
  const [filter, set] = useAtom<boolean>(filter_by_update_available);
  return [filter, () => set(!filter)];
};

export const usePermissions = ({ type, id }: Types.ResourceTarget) => {
  const user = useUser().data;
  const perms = useRead("GetPermission", { target: { type, id } }).data as
    | Types.PermissionLevelAndSpecifics
    | Types.PermissionLevel
    | undefined;
  const info = useRead("GetCoreInfo", {}).data;
  const ui_write_disabled = info?.ui_write_disabled ?? false;
  const disable_non_admin_create = info?.disable_non_admin_create ?? false;

  const level =
    (perms && typeof perms === "string" ? perms : perms?.level) ??
    Types.PermissionLevel.None;
  const specific =
    (perms && typeof perms === "string" ? [] : perms?.specific) ?? [];

  const canWrite = !ui_write_disabled && level === Types.PermissionLevel.Write;
  const canExecute = has_minimum_permissions(
    { level, specific },
    Types.PermissionLevel.Execute
  );

  const [
    specificLogs,
    specificInspect,
    specificTerminal,
    specificAttach,
    specificProcesses,
  ] = [
    specific.includes(Types.SpecificPermission.Logs),
    specific.includes(Types.SpecificPermission.Inspect),
    specific.includes(Types.SpecificPermission.Terminal),
    specific.includes(Types.SpecificPermission.Attach),
    specific.includes(Types.SpecificPermission.Processes),
  ];

  const canCreate =
    type === "Server"
      ? user?.admin ||
        (!disable_non_admin_create && user?.create_server_permissions)
      : type === "Build"
        ? user?.admin ||
          (!disable_non_admin_create && user?.create_build_permissions)
        : type === "Alerter" ||
            type === "Builder" ||
            type === "Procedure" ||
            type === "Action"
          ? user?.admin
          : user?.admin || !disable_non_admin_create;

  return {
    canWrite,
    canExecute,
    canCreate,
    specific,
    specificLogs,
    specificInspect,
    specificTerminal,
    specificAttach,
    specificProcesses,
  };
};

export const useTerminalTargetPermissions = (target: Types.TerminalTarget) => {
  const resourceTarget = resourceTargetFromTerminalTarget(target);
  return usePermissions(resourceTarget);
};

const templatesQueryBehaviorAtom =
  atomWithStorage<Types.TemplatesQueryBehavior>(
    "templates-query-behavior-v0",
    Types.TemplatesQueryBehavior.Exclude
  );

export const useTemplatesQueryBehavior = () =>
  useAtom<Types.TemplatesQueryBehavior>(templatesQueryBehaviorAtom);

export type SettingsView =
  | "Variables"
  | "Tags"
  | "Providers"
  | "Users"
  | "Profile";

const viewAtom = atomWithStorage<SettingsView>("settings-view-v2", "Variables");

export const useSettingsView = () => useAtom<SettingsView>(viewAtom);

/**
 * Map of unique host ports to array of formatted full port map spec
 * Formatted ex: 0.0.0.0:3000:3000/tcp
 */
export type PortsMap = { [host_port: string]: Array<Types.Port> };

export const useContainerPortsMap = (ports: Types.Port[]) => {
  return useMemo(() => {
    const map: PortsMap = {};
    for (const port of ports) {
      if (!port.PublicPort || !port.PrivatePort) continue;
      if (map[port.PublicPort]) {
        map[port.PublicPort].push(port);
      } else {
        map[port.PublicPort] = [port];
      }
    }
    for (const key in map) {
      map[key].sort();
    }
    return map;
  }, [ports]);
};

/**
 * A custom React hook that debounces a value, delaying its update until after
 * a specified period of inactivity. This is useful for performance optimization
 * in scenarios like search inputs, form validation, or API calls.
 */
export function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);

  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    return () => {
      clearTimeout(handler);
    };
  }, [value, delay]);

  return debouncedValue;
}
