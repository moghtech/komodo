import { KOMODO_BASE_URL } from "@/main";
import { KomodoClient, MoghAuth, Types } from "komodo_client";
import {
  type ExecuteResponses,
  type ReadResponses,
  type WriteResponses,
} from "komodo_client";
import {
  UseMutationOptions,
  UseQueryOptions,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { useCallback, useEffect, useMemo } from "react";
import { atom, useAtom } from "jotai";
import { atomFamily } from "jotai-family";
import { atomWithHash } from "jotai-location";
import { useParams } from "react-router-dom";
import { UsableResource, RESOURCE_TARGETS } from "@/resources";
import {
  hasMinimumPermissions,
  resourceTargetFromTerminalTarget,
} from "@/lib/utils";
import { notifications } from "@mantine/notifications";

export function komodo_client() {
  return KomodoClient(KOMODO_BASE_URL, {
    type: "jwt",
    params: { jwt: MoghAuth.LOGIN_TOKENS.jwt() },
  });
}

// ============== RESOLVER ==============

export function useUser() {
  const userReset = useUserReset();
  const hasJwt = !!MoghAuth.LOGIN_TOKENS.jwt();

  const query = useQuery({
    queryKey: ["GetUser"],
    queryFn: () => komodo_client().getUser(),
    refetchInterval: 30_000,
    enabled: hasJwt,
  });

  useEffect(() => {
    if (query.data && query.error) {
      userReset();
    }
  }, [query.data, query.error]);

  return query;
}

export function useUserInvalidate() {
  const qc = useQueryClient();
  return () => {
    qc.invalidateQueries({ queryKey: ["GetUser"] });
  };
}

export function useUserReset() {
  const qc = useQueryClient();
  return () => {
    qc.resetQueries({ queryKey: ["GetUser"] });
  };
}

export function useRead<
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
>(type: T, params: P, config?: C) {
  const hasJwt = !!MoghAuth.LOGIN_TOKENS.jwt();
  return useQuery({
    queryKey: [type, params],
    queryFn: () => komodo_client().read<T, R>(type, params),
    enabled: hasJwt && config?.enabled !== false,
    ...config,
  });
}

export function useInvalidate() {
  const qc = useQueryClient();
  return <
    Type extends Types.ReadRequest["type"],
    Params extends Extract<Types.ReadRequest, { type: Type }>["params"],
  >(
    ...keys: Array<[Type] | [Type, Params]>
  ) => keys.forEach((key) => qc.invalidateQueries({ queryKey: key }));
}

export function useWrite<
  T extends Types.WriteRequest["type"],
  R extends Extract<Types.WriteRequest, { type: T }>,
  P extends R["params"],
  C extends Omit<
    UseMutationOptions<WriteResponses[R["type"]], unknown, P, unknown>,
    "mutationKey" | "mutationFn"
  >,
>(type: T, config?: C) {
  return useMutation({
    mutationKey: [type],
    mutationFn: (params: P) => komodo_client().write<T, R>(type, params),
    onError: (e: { result: { error?: string; trace?: string[] } }, ...args) => {
      console.log("Write error:", e);
      const msg = e.result.error ?? "Unknown error. See console.";
      const detail = e.result?.trace
        ?.map((msg) => msg[0].toUpperCase() + msg.slice(1))
        .join(" | ");
      let msg_log = msg ? msg[0].toUpperCase() + msg.slice(1) + " | " : "";
      if (detail) {
        msg_log += detail + " | ";
      }
      notifications.show({
        title: `Write request ${type} failed`,
        message: `${msg_log}See console for details`,
        color: "red",
      });
      config?.onError && config.onError(e, ...args);
    },
    ...config,
  });
}

export function useExecute<
  T extends Types.ExecuteRequest["type"],
  R extends Extract<Types.ExecuteRequest, { type: T }>,
  P extends R["params"],
  C extends Omit<
    UseMutationOptions<ExecuteResponses[T], unknown, P, unknown>,
    "mutationKey" | "mutationFn"
  >,
>(type: T, config?: C) {
  return useMutation({
    mutationKey: [type],
    mutationFn: (params: P) => komodo_client().execute<T, R>(type, params),
    onError: (e: { result: { error?: string; trace?: string[] } }, ...args) => {
      console.log("Execute error:", e);
      const msg = e.result.error ?? "Unknown error. See console.";
      const detail = e.result?.trace
        ?.map((msg) => msg[0].toUpperCase() + msg.slice(1))
        .join(" | ");
      let msg_log = msg ? msg[0].toUpperCase() + msg.slice(1) + " | " : "";
      if (detail) {
        msg_log += detail + " | ";
      }
      notifications.show({
        title: `Execute request ${type} failed`,
        message: `${msg_log}See console for details`,
        color: "red",
      });
      config?.onError && config.onError(e, ...args);
    },
    ...config,
  });
}

// ============== UTILITY ==============

export function atomWithStorage<T>(key: string, init: T) {
  const stored = localStorage.getItem(key);
  const inner = atom(stored ? JSON.parse(stored) : init);
  return atom(
    (get) => get(inner),
    (_, set, newValue) => {
      set(inner, newValue);
      localStorage.setItem(key, JSON.stringify(newValue));
    },
  );
}

export function useResourceName(type: UsableResource) {
  const resources = useRead(`List${type}s`, {}).data;
  return useCallback(
    (id: string) => resources?.find((resource) => resource.id === id)?.name,
    [resources],
  );
}

export function useResourceParamType() {
  const type = useParams().type;
  if (!type) return undefined;
  if (type === "resource-syncs") return "ResourceSync";
  return (type[0].toUpperCase() + type.slice(1, -1)) as UsableResource;
}

export type ResourceMap = {
  [Resource in UsableResource]: Types.ResourceListItem<unknown>[] | undefined;
};

export function useAllResources(refetchInterval?: number): ResourceMap {
  return {
    Swarm: useRead("ListSwarms", {}, { refetchInterval }).data,
    Server: useRead("ListServers", {}, { refetchInterval }).data,
    Stack: useRead("ListStacks", {}, { refetchInterval }).data,
    Deployment: useRead("ListDeployments", {}, { refetchInterval }).data,
    Build: useRead("ListBuilds", {}, { refetchInterval }).data,
    Repo: useRead("ListRepos", {}, { refetchInterval }).data,
    Procedure: useRead("ListProcedures", {}, { refetchInterval }).data,
    Action: useRead("ListActions", {}, { refetchInterval }).data,
    Builder: useRead("ListBuilders", {}, { refetchInterval }).data,
    Alerter: useRead("ListAlerters", {}, { refetchInterval }).data,
    ResourceSync: useRead("ListResourceSyncs", {}, { refetchInterval }).data,
  };
}

// Returns true if Komodo has no resources.
export function useNoResources() {
  const resources = useAllResources();
  for (const target of RESOURCE_TARGETS) {
    if (resources[target] && resources[target].length) {
      return false;
    }
  }
  return true;
}

/** returns function that takes a resource target and checks if it exists */
export function useCheckResourceExists() {
  const resources = useAllResources();
  return (target: Types.ResourceTarget) => {
    return (
      resources[target.type as UsableResource]?.some(
        (resource) => resource.id === target.id,
      ) || false
    );
  };
}

export function useFilterResources<Info>(
  resources?: Types.ResourceListItem<Info>[],
  search?: string,
) {
  const tags = useTagsFilter();
  const searchSplit = search?.toLowerCase()?.split(" ") || [];
  return (
    resources?.filter(
      (resource) =>
        tags.every((tag: string) => resource.tags.includes(tag)) &&
        (searchSplit.length > 0
          ? searchSplit.every((search) =>
              resource.name.toLowerCase().includes(search),
            )
          : true),
    ) ?? []
  );
}

export function usePushRecentlyViewed({ type, id }: Types.ResourceTarget) {
  const userInvalidate = useUserInvalidate();

  const push = useWrite("PushRecentlyViewed", {
    onSuccess: userInvalidate,
  }).mutate;

  const exists = useRead(`List${type as UsableResource}s`, {}).data?.find(
    (r) => r.id === id,
  )
    ? true
    : false;

  useEffect(() => {
    exists && push({ resource: { type, id } });
  }, [exists, push]);

  return () => push({ resource: { type, id } });
}

export function useSetTitle(more?: string) {
  const info = useRead("GetCoreInfo", {}).data;
  const title = more ? `${more} | ${info?.title}` : info?.title;
  useEffect(() => {
    if (title) {
      document.title = title;
    }
  }, [title]);
}

const tagsAtom = atomWithStorage<string[]>("tags-v0", []);

export function useTags() {
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
}

export function useTagsFilter() {
  const [tags] = useAtom<string[]>(tagsAtom);
  return tags;
}

export type WebhookIntegration = "Github" | "Gitlab";
export type WebhookIntegrations = {
  [key: string]: WebhookIntegration;
};

const WEBHOOK_INTEGRATIONS_ATOM = atomWithStorage<WebhookIntegrations>(
  "webhook-integrations-v2",
  {},
);

export function useWebhookIntegrations() {
  const [integrations, setIntegrations] = useAtom<WebhookIntegrations>(
    WEBHOOK_INTEGRATIONS_ATOM,
  );
  return {
    integrations,
    setIntegration: (provider: string, integration: WebhookIntegration) =>
      setIntegrations({
        ...integrations,
        [provider]: integration,
      }),
    getIntegration: (provider: string) => {
      return integrations[provider]
        ? integrations[provider]
        : provider.includes("gitlab")
          ? "Gitlab"
          : "Github";
    },
  };
}

export type WebhookIdOrName = "Id" | "Name";

const WEBHOOK_ID_OR_NAME_ATOM = atomWithStorage<WebhookIdOrName>(
  "webhook-id-or-name-v1",
  "Id",
);

export function useWebhookIdOrName() {
  return useAtom<WebhookIdOrName>(WEBHOOK_ID_OR_NAME_ATOM);
}

const selectedResources = atomFamily((_: UsableResource) => atom<string[]>([]));
export function useSelectedResources(type: UsableResource) {
  return useAtom(selectedResources(type));
}

const filterByUpdateAvailable = atomWithHash<boolean>(
  "filter-update-available",
  false,
);
export function useFilterByUpdateAvailable(): [boolean, () => void] {
  const [filter, set] = useAtom<boolean>(filterByUpdateAvailable);
  return [filter, () => set(!filter)];
}

export function usePermissions({ type, id }: Types.ResourceTarget) {
  const user = useUser().data;
  const perms = useRead("GetPermission", { target: { type, id } }).data as
    | Types.PermissionLevelAndSpecifics
    | Types.PermissionLevel
    | undefined;
  const info = useRead("GetCoreInfo", {}).data;
  const ui_write_disabled = info?.ui_write_disabled ?? false;
  const disable_non_admin_create = info?.disable_non_admin_create ?? false;

  const level =
    (perms && typeof perms === "string"
      ? perms
      : (perms as Types.PermissionLevelAndSpecifics | undefined)?.level) ??
    Types.PermissionLevel.None;
  const specific =
    (perms && typeof perms === "string"
      ? []
      : (perms as Types.PermissionLevelAndSpecifics | undefined)?.specific) ??
    [];

  const canWrite = !ui_write_disabled && level === Types.PermissionLevel.Write;
  const canExecute = hasMinimumPermissions(
    { level, specific },
    Types.PermissionLevel.Execute,
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
}

export function useTerminalTargetPermissions(target: Types.TerminalTarget) {
  const resourceTarget = resourceTargetFromTerminalTarget(target);
  return usePermissions(resourceTarget);
}

const templatesQueryBehaviorAtom =
  atomWithStorage<Types.TemplatesQueryBehavior>(
    "templates-query-behavior-v0",
    Types.TemplatesQueryBehavior.Exclude,
  );

export function useTemplatesQueryBehavior() {
  return useAtom<Types.TemplatesQueryBehavior>(templatesQueryBehaviorAtom);
}

export type SettingsView =
  | "Variables"
  | "Tags"
  | "Builders"
  | "Alerters"
  | "Providers"
  | "Users"
  | "Onboarding";

const viewAtom = atomWithStorage<SettingsView>("settings-view-v2", "Variables");
export function useSettingsView() {
  return useAtom<SettingsView>(viewAtom);
}

/**
 * Map of unique host ports to array of formatted full port map spec
 * Formatted ex: 0.0.0.0:3000:3000/tcp
 */
export type PortsMap = { [host_port: string]: Array<Types.Port> };

export function useContainerPortsMap(ports: Types.Port[]) {
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
}

interface DashboardPreferences {
  showServerStats: boolean;
  showTables: boolean;
}

const dashboardPreferencesAtom = atomWithStorage(
  "komodo-dashboard-preferences-v2",
  {
    showServerStats: false,
    showTables: false,
  },
);

export function useDashboardPreferences() {
  const [preferences, setPreferences] = useAtom<DashboardPreferences>(
    dashboardPreferencesAtom,
  );

  const updatePreference = <K extends keyof DashboardPreferences>(
    key: K,
    value: DashboardPreferences[K],
  ) => {
    setPreferences({ ...preferences, [key]: value });
  };

  const togglePreference = <K extends keyof DashboardPreferences>(key: K) => {
    updatePreference(key, !preferences[key]);
  };

  return {
    preferences,
    updatePreference,
    togglePreference,
  };
}

export function useUserTargetPermissions(user_target: Types.UserTarget) {
  const permissions = useRead("ListUserTargetPermissions", {
    user_target,
  }).data;
  const allResources = useAllResources();
  const perms: (Types.Permission & { name: string })[] = [];
  for (const [resource_type, resources] of Object.entries(allResources)) {
    addUserTargetPermissions(
      user_target,
      permissions,
      resource_type as UsableResource,
      resources,
      perms,
    );
  }
  return perms;
}

function addUserTargetPermissions<I>(
  user_target: Types.UserTarget,
  permissions: Types.Permission[] | undefined,
  resource_type: UsableResource,
  resources: Types.ResourceListItem<I>[] | undefined,
  perms: (Types.Permission & { name: string })[],
) {
  resources?.forEach((resource) => {
    const perm = permissions?.find(
      (p) =>
        p.resource_target.type === resource_type &&
        p.resource_target.id === resource.id,
    );
    if (perm) {
      perms.push({ ...perm, name: resource.name });
    } else {
      perms.push({
        user_target,
        name: resource.name,
        level: Types.PermissionLevel.None,
        resource_target: { type: resource_type, id: resource.id },
      });
    }
  });
}
