import { UsableResource } from "@/resources";
import { Types } from "komodo_client";
import sanitizeHtml from "sanitize-html";
import ConvertAnsiToHtml from "ansi-to-html";
import { notifications } from "@mantine/notifications";

export function sanitizeQuery() {
  sanitizeQueryInner(new URLSearchParams(location.search));
}

export function sanitizeQueryInner(search: URLSearchParams) {
  search.delete("redeem_ready");
  search.delete("totp");
  search.delete("passkey");
  const query = search.toString();
  location.replace(
    `${location.origin}${location.pathname}${query.length ? "?" + query : ""}`,
  );
}

export function objectKeys<T extends object>(o: T): (keyof T)[] {
  return Object.keys(o) as (keyof T)[];
}

export function envToText(envVars: Types.EnvironmentVar[] | undefined) {
  return envVars?.reduce(
    (prev, { variable, value }) =>
      prev + (prev ? "\n" : "") + `${variable}: ${value}`,
    "",
  );
}

export function recordToText(record: Record<string, string> | undefined) {
  return Object.entries(record ?? {})
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([key, value]) => `${key}: ${value}`)
    .join("\n");
}

export function textToEnv(env: string): Types.EnvironmentVar[] {
  return env
    .split("\n")
    .filter((line) => keepLine(line))
    .map((entry) => {
      const [first, ...rest] = entry.replaceAll('"', "").split("=");
      return [first, rest.join("=")];
    })
    .map(([variable, value]) => ({ variable, value }));
}

export function textToRecord(text: string): Record<string, string> {
  const record: Record<string, string> = {};

  for (const rawLine of text.split("\n")) {
    if (!keepLine(rawLine)) continue;

    const line = rawLine
      .split(" #", 1)[0]
      .trim()
      .replace(/^-/, "")
      .trim();
    const separator = line.search(/[:=]/);

    if (separator === -1) {
      throw new Error("Each header line must contain ':' or '='.");
    }

    const key = line.slice(0, separator).trim().replace(/^["']|["']$/g, "");
    const value = line
      .slice(separator + 1)
      .trim()
      .replace(/^["']|["']$/g, "");

    if (!key) {
      throw new Error("Header names cannot be empty.");
    }

    record[key] = value;
  }

  return record;
}

function keepLine(line: string) {
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

export function parseKeyValue(
  input: string,
): Array<{ key: string; value: string }> {
  const trimmed = input.trim();
  if (trimmed.length === 0) return [];
  return trimmed
    .split("\n")
    .map((line) => line.trim())
    .filter(
      (line) =>
        line.length > 0 && !line.startsWith("#") && !line.startsWith("//"),
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

export function versionIsNone(version?: Types.Version) {
  if (!version) return true;
  return version.major === 0 && version.minor === 0 && version.patch === 0;
}

export function levelToNumber(level: Types.PermissionLevel | undefined) {
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
}

export function levelSortingFn(
  a: Types.PermissionLevel | undefined,
  b: Types.PermissionLevel | undefined,
) {
  const al = levelToNumber(a);
  const bl = levelToNumber(b);
  const dif = al - bl;
  return dif === 0 ? 0 : dif / Math.abs(dif);
}

export function hasMinimumPermissions(
  permission: Types.PermissionLevelAndSpecifics | undefined,
  greater_than: Types.PermissionLevel,
  specific?: Types.SpecificPermission[],
) {
  if (!permission) return false;
  if (levelToNumber(permission.level) < levelToNumber(greater_than))
    return false;
  if (!specific) return true;
  for (const s of specific) {
    if (!permission.specific.includes(s)) {
      return false;
    }
  }
  return true;
}

export function usableResourcePath(resource: UsableResource) {
  if (resource === "ResourceSync") return "resource-syncs";
  return `${resource.toLowerCase()}s`;
}

export function usableResourceExecuteKey(resource: UsableResource) {
  if (resource === "ResourceSync") return "sync";
  return `${resource.toLowerCase()}`;
}

export function sanitizeOnlySpan(log: string) {
  return sanitizeHtml(log, {
    allowedTags: ["span"],
    allowedAttributes: {
      span: ["style"],
    },
  });
}

const convert_ansi = new ConvertAnsiToHtml();

/**
 * Converts the ansi colors in an Update log to html.
 * sanitizes incoming log first for any eg. script tags.
 * @param log incoming log string
 */
export function updateLogToHtml(log: string) {
  if (!log) return "No log.";
  return convert_ansi.toHtml(sanitizeOnlySpan(log));
}

/**
 * Converts the ansi colors in log to html.
 * sanitizes incoming log first for any eg. script tags.
 * @param log incoming log string
 */
export function logToHtml(log: string) {
  if (!log) return "No log.";
  const sanitized = sanitizeHtml(log, {
    allowedTags: sanitizeHtml.defaults.allowedTags.filter(
      (tag) => tag !== "script",
    ),
    allowedAttributes: sanitizeHtml.defaults.allowedAttributes,
  });
  return convert_ansi.toHtml(sanitized);
}

export function getUpdateQuery(
  target: Types.ResourceTarget,
  deployments: Types.DeploymentListItem[] | undefined,
) {
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
}

export function filterBySplit<T>(
  items: T[] | undefined,
  search: string,
  extract: (item: T) => string,
) {
  const split = search.toLowerCase().split(" ");
  return (
    (split.length
      ? items?.filter((item) => {
          const target = extract(item).toLowerCase();
          return split.every((term) => target.includes(term));
        })
      : items) ?? []
  );
}

export function filterMultitermBySplit<T>(
  items: T[] | undefined,
  search: string,
  extract: (item: T) => (string | undefined)[],
) {
  const split = search.toLowerCase().split(" ");
  return (
    (split.length
      ? items?.filter((item) => {
          const target = extract(item)
            .filter((str) => str)
            .map((str) => str!.toLowerCase());
          return split.every(
            (term) => target.findIndex((t) => t.includes(term)) !== -1,
          );
        })
      : items) ?? []
  );
}

/** This does NOT include pending deploys, which are only for Execute direction. */
export function resourceSyncNoChanges(sync: Types.ResourceSync) {
  return (
    (sync.info?.resource_updates?.length ?? 0) === 0 &&
    (sync.info?.variable_updates?.length ?? 0) === 0 &&
    (sync.info?.user_group_updates?.length ?? 0) === 0
  );
}

export function extractRegistryDomain(image_name: string) {
  if (!image_name) return "docker.io";
  const maybe_domain = image_name.split("/")[0];
  if (maybe_domain.includes(".")) {
    return maybe_domain;
  } else {
    return "docker.io";
  }
}

/** Checks file contents empty, not including whitespace / comments */
export function fileContentsEmpty(contents?: string) {
  if (!contents) return true;
  return (
    contents
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line.length !== 0 && !line.startsWith("#")).length === 0
  );
}

export function resourceTargetFromTerminalTarget(
  target: Types.TerminalTarget,
): Types.ResourceTarget {
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
}

export function terminalLink({
  target,
  name,
}: {
  target: Types.TerminalTarget;
  name: string;
}) {
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
}

export function sendCopyNotification(label = "content") {
  if (location.origin.startsWith("https")) {
    notifications.show({
      message: `Copied ${label} to clipboard.`,
      color: "green",
    });
  } else {
    notifications.show({
      message: "Cannot copy to clipboard without HTTPS.",
      color: "red",
    });
  }
}

export function listsEqual(a: string[], b: string[]) {
  for (const aa of a) {
    if (!b.includes(aa)) {
      return false;
    }
  }
  for (const bb of b) {
    if (!a.includes(bb)) {
      return false;
    }
  }
  return true;
}

/**
 * Does deep compare of 2 items, returning `true` if equal.
 *
 * - Functions: Always `true`
 * - Primitives: Returns direct `a === b`
 * - Arrays: Returns same items and ordering (recursive)
 * - Objects: Returns same keys / values (recursive)
 *
 * @param a Item a
 * @param b Item b
 * @returns a === b
 */
export function deepCompare(a: any, b: any) {
  // Short path for falsy. Important to catch typeof null === "object" edge case.
  if (!a || !b) {
    return a === b;
  }

  const ta = typeof a;
  const tb = typeof b;

  if (ta !== tb) return false;

  if (ta === "function") return true;

  if (ta === "object") {
    const ea = Object.entries(a);
    const kb = Object.keys(b);

    // Length not equal -> false
    if (ea.length !== kb.length) return false;

    for (const [key, va] of ea) {
      const vb = b[key];

      // Early return when any not equal
      if (!deepCompare(va, vb)) return false;
    }

    // If it gets through all, it's equal
    return true;
  }

  return a === b;
}
