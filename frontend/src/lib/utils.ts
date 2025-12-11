import { ResourceComponents } from "@components/resources";
import { Types } from "komodo_client";
import { UsableResource } from "@types";
import Convert from "ansi-to-html";
import { type ClassValue, clsx } from "clsx";
import sanitizeHtml from "sanitize-html";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export const object_keys = <T extends object>(o: T): (keyof T)[] =>
  Object.keys(o) as (keyof T)[];

export const RESOURCE_TARGETS: UsableResource[] = [
  "Swarm",
  "Server",
  "Stack",
  "Deployment",
  "Build",
  "Repo",
  "Procedure",
  "Action",
  "Builder",
  "Alerter",
  "ResourceSync",
];

export const SETTINGS_RESOURCES: UsableResource[] = ["Builder", "Alerter"];

export const SIDEBAR_RESOURCES: UsableResource[] = RESOURCE_TARGETS.filter(
  (target) => !SETTINGS_RESOURCES.includes(target)
);

export function env_to_text(envVars: Types.EnvironmentVar[] | undefined) {
  return envVars?.reduce(
    (prev, { variable, value }) =>
      prev + (prev ? "\n" : "") + `${variable}: ${value}`,
    ""
  );
}

export function text_to_env(env: string): Types.EnvironmentVar[] {
  return env
    .split("\n")
    .filter((line) => keep_line(line))
    .map((entry) => {
      const [first, ...rest] = entry.replaceAll('"', "").split("=");
      return [first, rest.join("=")];
    })
    .map(([variable, value]) => ({ variable, value }));
}

function keep_line(line: string) {
  if (line.length === 0) return false;
  let firstIndex = -1;
  for (let i = 0; i < line.length; i++) {
    if (line[i] !== " ") {
      firstIndex = i;
      break;
    }
  }
  if (firstIndex === -1) return false;
  if (line[firstIndex] === "#") return false;
  return true;
}

export function parse_key_value(
  input: string
): Array<{ key: string; value: string }> {
  const trimmed = input.trim();
  if (trimmed.length === 0) return [];
  return trimmed
    .split("\n")
    .map((line) => line.trim())
    .filter(
      (line) =>
        line.length > 0 && !line.startsWith("#") && !line.startsWith("//")
    )
    .map((line) => {
      const no_comment = line.split(" #", 1)[0].trim();
      const no_dash = no_comment.startsWith("-")
        ? no_comment.slice(1).trim()
        : no_comment;
      const no_leading_quote = no_dash.startsWith('"')
        ? no_dash.slice(1)
        : no_dash;
      const no_trailing_quote = no_leading_quote.endsWith('"')
        ? no_leading_quote.slice(0, -1)
        : no_leading_quote;
      const res = no_trailing_quote.split(/[=: ]/, 1);
      const [key, value] = [res[0]?.trim() ?? "", res[1]?.trim() ?? ""];
      const value_no_leading_quote = value.startsWith('"')
        ? value.slice(1)
        : value;
      const value_no_trailing_quote = value_no_leading_quote.endsWith('"')
        ? value_no_leading_quote.slice(0, -1)
        : value_no_leading_quote;
      return { key, value: value_no_trailing_quote.trim() };
    });
}

export function version_is_none(version?: Types.Version) {
  if (!version) return true;
  return version.major === 0 && version.minor === 0 && version.patch === 0;
}

export function resource_name(type: UsableResource, id: string) {
  const Components = ResourceComponents[type];
  return Components.list_item(id)?.name;
}

export const level_to_number = (level: Types.PermissionLevel | undefined) => {
  switch (level) {
    case undefined:
      return 0;
    case Types.PermissionLevel.None:
      return 0;
    case Types.PermissionLevel.Read:
      return 1;
    case Types.PermissionLevel.Execute:
      return 2;
    case Types.PermissionLevel.Write:
      return 3;
  }
};

export const has_minimum_permissions = (
  permission: Types.PermissionLevelAndSpecifics | undefined,
  greater_than: Types.PermissionLevel,
  specific?: Types.SpecificPermission[]
) => {
  if (!permission) return false;
  if (level_to_number(permission.level) < level_to_number(greater_than))
    return false;
  if (!specific) return true;
  for (const s of specific) {
    if (!permission.specific.includes(s)) {
      return false;
    }
  }
  return true;
};

const tzOffsetMs = new Date().getTimezoneOffset() * 60 * 1000;

export const convertTsMsToLocalUnixTsInMs = (ts: number) => ts - tzOffsetMs;

export const usableResourcePath = (resource: UsableResource) => {
  if (resource === "ResourceSync") return "resource-syncs";
  return `${resource.toLowerCase()}s`;
};

export const usableResourceExecuteKey = (resource: UsableResource) => {
  if (resource === "ResourceSync") return "sync";
  return `${resource.toLowerCase()}`;
};

export const sanitizeOnlySpan = (log: string) => {
  return sanitizeHtml(log, {
    allowedTags: ["span"],
    allowedAttributes: {
      span: ["class"],
    },
  });
};

/**
 * Converts the ansi colors in an Update log to html.
 * sanitizes incoming log first for any eg. script tags.
 * @param log incoming log string
 */
export const updateLogToHtml = (log: string) => {
  if (!log) return "No log.";
  return convert.toHtml(sanitizeOnlySpan(log));
};

const convert = new Convert();
/**
 * Converts the ansi colors in log to html.
 * sanitizes incoming log first for any eg. script tags.
 * @param log incoming log string
 */
export const logToHtml = (log: string) => {
  if (!log) return "No log.";
  const sanitized = sanitizeHtml(log, {
    allowedTags: sanitizeHtml.defaults.allowedTags.filter(
      (tag) => tag !== "script"
    ),
    allowedAttributes: sanitizeHtml.defaults.allowedAttributes,
  });
  return convert.toHtml(sanitized);
};

export const getUpdateQuery = (
  target: Types.ResourceTarget,
  deployments: Types.DeploymentListItem[] | undefined
) => {
  const build_id =
    target.type === "Deployment"
      ? deployments?.find((d) => d.id === target.id)?.info.build_id
      : undefined;
  if (build_id) {
    return {
      $or: [
        {
          "target.type": target.type,
          "target.id": target.id,
        },
        {
          "target.type": "Build",
          "target.id": build_id,
          operation: {
            $in: [Types.Operation.RunBuild, Types.Operation.CancelBuild],
          },
        },
      ],
    };
  } else {
    return {
      "target.type": target.type,
      "target.id": target.id,
    };
  }
};

export const filterBySplit = <T>(
  items: T[] | undefined,
  search: string,
  extract: (item: T) => string
) => {
  const split = search.toLowerCase().split(" ");
  return (
    (split.length
      ? items?.filter((item) => {
          const target = extract(item).toLowerCase();
          return split.every((term) => target.includes(term));
        })
      : items) ?? []
  );
};

export const sync_no_changes = (sync: Types.ResourceSync) => {
  return (
    (sync.info?.pending_deploy?.to_deploy ?? 0) === 0 &&
    (sync.info?.resource_updates?.length ?? 0) === 0 &&
    (sync.info?.variable_updates?.length ?? 0) === 0 &&
    (sync.info?.user_group_updates?.length ?? 0) === 0
  );
};

export const extract_registry_domain = (image_name: string) => {
  if (!image_name) return "docker.io";
  const maybe_domain = image_name.split("/")[0];
  if (maybe_domain.includes(".")) {
    return maybe_domain;
  } else {
    return "docker.io";
  }
};

/** Checks file contents empty, not including whitespace / comments */
export const file_contents_empty = (contents?: string) => {
  if (!contents) return true;
  return (
    contents
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line.length !== 0 && !line.startsWith("#")).length === 0
  );
};

export const resourceTargetFromTerminalTarget = (
  target: Types.TerminalTarget
): Types.ResourceTarget => {
  switch (target.type) {
    case "Server":
      return { type: "Server", id: target.params.server! };
    case "Container":
      return { type: "Server", id: target.params.server };
    case "Stack":
      return { type: "Stack", id: target.params.stack };
    case "Deployment":
      return { type: "Deployment", id: target.params.deployment };
  }
};

export const terminalLink = ({
  target,
  name,
}: {
  target: Types.TerminalTarget;
  name: string;
}) => {
  switch (target.type) {
    case "Server":
      return `/servers/${target.params.server}/terminal/${name}`;
    case "Container":
      return `/servers/${target.params.server}/container/${target.params.container}/terminal/${name}`;
    case "Stack":
      return `/stacks/${target.params.stack}/service/${target.params.service}/terminal/${name}`;
    case "Deployment":
      return `/deployments/${target.params.deployment}/terminal/${name}`;
  }
};

export const base64urlToArrayBuffer = (base64url: any) => {
  // Convert from URL-safe base64 to normal base64
  const base64 = base64url.replace(/-/g, "+").replace(/_/g, "/");
  const pad =
    base64.length % 4 === 0 ? "" : "=".repeat(4 - (base64.length % 4));
  const bstr = atob(base64 + pad);
  const bytes = new Uint8Array(bstr.length);
  for (let i = 0; i < bstr.length; i++) {
    bytes[i] = bstr.charCodeAt(i);
  }
  return bytes.buffer;
};

export const arrayBufferToBase64url = (buffer: any) => {
  const bytes = new Uint8Array(buffer);
  let binary = "";
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  const base64 = btoa(binary);
  return base64.replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
};

export const preparePasskeyCredential = (data: any) => {
  return {
    ...data,
    publicKey: {
      ...data.publicKey,
      challenge: base64urlToArrayBuffer(data.publicKey.challenge),
      allowCredentials: data.publicKey.allowCredentials?.map((cred: any) => ({
        ...cred,
        id: base64urlToArrayBuffer(cred.id),
      })),
    },
  };
};
