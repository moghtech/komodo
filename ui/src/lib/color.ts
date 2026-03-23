import { Types } from "komodo_client";

export type ColorIntention =
  | "Good"
  | "Neutral"
  | "Warning"
  | "Critical"
  | "Unknown"
  | "None";

export const colorByIntention = (intention: ColorIntention) => {
  switch (intention) {
    case "Good":
      return "green";
    case "Neutral":
      return "blue";
    case "Warning":
      return "yellow";
    case "Critical":
      return "red";
    case "Unknown":
      return "purple";
    case "None":
      return undefined;
  }
};

export const hexColorByIntention = (intention: ColorIntention) => {
  switch (intention) {
    case "Good":
      return "#22C55E";
    case "Neutral":
      return "#3B82F6";
    case "Warning":
      return "#EAB308";
    case "Critical":
      return "#EF0044";
    case "Unknown":
      return "#A855F7";
    case "None":
      return undefined;
  }
};

export const swarmStateIntention: (
  state?: Types.SwarmState,
) => ColorIntention = (state) => {
  switch (state) {
    case Types.SwarmState.Healthy:
      return "Good";
    case Types.SwarmState.Unhealthy:
      return "Critical";
    case Types.SwarmState.Down:
      return "Neutral";
    case Types.SwarmState.Unknown:
      return "Unknown";
    case undefined:
      return "None";
  }
};

export const swarmNodeStateIntention: (
  state?: Types.NodeState,
) => ColorIntention = (state) => {
  switch (state) {
    case Types.NodeState.READY:
      return "Good";
    case Types.NodeState.DOWN:
      return "Critical";
    case Types.NodeState.DISCONNECTED:
      return "Critical";
    case Types.NodeState.UNKNOWN:
      return "Neutral";
    case undefined:
      return "None";
  }
};

export const swarmNodeAvailabilityIntention: (
  state?: Types.NodeSpecAvailabilityEnum,
) => ColorIntention = (state) => {
  switch (state) {
    case Types.NodeSpecAvailabilityEnum.ACTIVE:
      return "Good";
    case Types.NodeSpecAvailabilityEnum.DRAIN:
      return "Warning";
    case Types.NodeSpecAvailabilityEnum.PAUSE:
      return "Critical";
    case Types.NodeSpecAvailabilityEnum.EMPTY:
      return "Neutral";
    case undefined:
      return "None";
  }
};

export const swarmNodeRoleIntention: (
  state?: Types.NodeSpecRoleEnum,
) => ColorIntention = (state) => {
  switch (state) {
    case Types.NodeSpecRoleEnum.MANAGER:
      return "Good";
    case Types.NodeSpecRoleEnum.WORKER:
      return "Neutral";
    case Types.NodeSpecRoleEnum.EMPTY:
      return "None";
    case undefined:
      return "None";
  }
};

export const swarmTaskStateIntention: (
  state?: Types.TaskState,
  desired?: Types.TaskState,
) => ColorIntention = (state, desired) => {
  // Case when its desired running
  if (desired === Types.TaskState.RUNNING) {
    if (state === Types.TaskState.RUNNING) {
      return "Good";
    } else {
      return "Critical";
    }
  }

  // Case when its desired shutdown
  if (desired === Types.TaskState.SHUTDOWN) {
    // If you want it shutdown, then running is critical.
    if (state === Types.TaskState.RUNNING) {
      return "Critical";
    } else {
      // Otherwise, it is "Down", give neutral color
      return "Neutral";
    }
  }

  // Others
  if (state === desired) {
    return "Good";
  } else {
    return "Critical";
  }
};

export const serverStateIntention: (
  state: Types.ServerState | undefined,
  versionMismatch: boolean,
) => ColorIntention = (state, versionMismatch) => {
  switch (state) {
    case Types.ServerState.Ok:
      // If there's a version mismatch and the server is "Ok", show warning instead
      return versionMismatch ? "Warning" : "Good";
    case Types.ServerState.NotOk:
      return "Critical";
    case Types.ServerState.Disabled:
      return "Neutral";
    case undefined:
      return "None";
  }
};

export const stackStateIntention = (
  state: Types.StackState | undefined,
  updateAvailable: boolean | undefined,
) => {
  switch (state) {
    case undefined:
      return "None";
    case Types.StackState.Deploying:
      return "Warning";
    case Types.StackState.Running:
      return updateAvailable ? "Warning" : "Good";
    case Types.StackState.Restarting:
      return "Critical";
    case Types.StackState.Down:
      return "Neutral";
    case Types.StackState.Unknown:
      return "Unknown";
    default:
      return "Critical";
  }
};

export const deploymentStateIntention: (
  state: Types.DeploymentState | undefined,
  updateAvailable: boolean | undefined,
) => ColorIntention = (state) => {
  switch (state) {
    case undefined:
      return "None";
    case Types.DeploymentState.Deploying:
      return "Warning";
    case Types.DeploymentState.Running:
      return "Good";
    case Types.DeploymentState.NotDeployed:
      return "Neutral";
    case Types.DeploymentState.Unknown:
      return "Unknown";
    default:
      return "Critical";
  }
};

export const containerStateIntention: (
  state?: Types.ContainerStateStatusEnum,
) => ColorIntention = (state) => {
  switch (state) {
    case undefined:
      return "None";
    case Types.ContainerStateStatusEnum.Running:
      return "Good";
    case Types.ContainerStateStatusEnum.Paused:
      return "Warning";
    case Types.ContainerStateStatusEnum.Empty:
      return "Unknown";
    default:
      return "Critical";
  }
};

export const buildStateIntention = (status?: Types.BuildState) => {
  switch (status) {
    case undefined:
      return "None";
    case Types.BuildState.Unknown:
      return "Unknown";
    case Types.BuildState.Ok:
      return "Good";
    case Types.BuildState.Building:
      return "Warning";
    case Types.BuildState.Failed:
      return "Critical";
    default:
      return "None";
  }
};

export const repoStateIntention = (state?: Types.RepoState) => {
  switch (state) {
    case undefined:
      return "None";
    case Types.RepoState.Unknown:
      return "Unknown";
    case Types.RepoState.Ok:
      return "Good";
    case Types.RepoState.Cloning:
      return "Warning";
    case Types.RepoState.Pulling:
      return "Warning";
    case Types.RepoState.Building:
      return "Warning";
    case Types.RepoState.Failed:
      return "Critical";
    default:
      return "None";
  }
};

export const procedureStateIntention = (status?: Types.ProcedureState) => {
  switch (status) {
    case undefined:
      return "None";
    case Types.ProcedureState.Unknown:
      return "Unknown";
    case Types.ProcedureState.Ok:
      return "Good";
    case Types.ProcedureState.Running:
      return "Warning";
    case Types.ProcedureState.Failed:
      return "Critical";
    default:
      return "None";
  }
};

export const actionStateIntention = (status?: Types.ActionState) => {
  switch (status) {
    case undefined:
      return "None";
    case Types.ActionState.Unknown:
      return "Unknown";
    case Types.ActionState.Ok:
      return "Good";
    case Types.ActionState.Running:
      return "Warning";
    case Types.ActionState.Failed:
      return "Critical";
    default:
      return "None";
  }
};

export const resourceSyncStateIntention = (
  status?: Types.ResourceSyncState,
) => {
  switch (status) {
    case undefined:
      return "None";
    case Types.ResourceSyncState.Unknown:
      return "Unknown";
    case Types.ResourceSyncState.Ok:
      return "Good";
    case Types.ResourceSyncState.Syncing:
      return "Warning";
    case Types.ResourceSyncState.Pending:
      return "Warning";
    case Types.ResourceSyncState.Failed:
      return "Critical";
    default:
      return "None";
  }
};

export const alertLevelIntention: (
  level: Types.SeverityLevel,
) => ColorIntention = (level) => {
  switch (level) {
    case Types.SeverityLevel.Ok:
      return "Good";
    case Types.SeverityLevel.Warning:
      return "Warning";
    case Types.SeverityLevel.Critical:
      return "Critical";
  }
};

export const diffTypeIntention: (
  level: Types.DiffData["type"],
  reverse: boolean,
) => ColorIntention = (level, reverse) => {
  switch (level) {
    case "Create":
      return reverse ? "Critical" : "Good";
    case "Update":
      return "Neutral";
    case "Delete":
      return reverse ? "Good" : "Critical";
  }
};

export const tagColor = (color?: Types.TagColor) => {
  switch (color) {
    case undefined:
      return "#475569"; // slate-600
    case Types.TagColor.LightSlate:
      return "#94A3B8"; // slate-400
    case Types.TagColor.Slate:
      return "#475569"; // slate-600
    case Types.TagColor.DarkSlate:
      return "#0F172A"; // slate-900

    case Types.TagColor.LightRed:
      return "#F87171"; // red-400
    case Types.TagColor.Red:
      return "#DC2626"; // red-600
    case Types.TagColor.DarkRed:
      return "#7F1D1D"; // red-900

    case Types.TagColor.LightOrange:
      return "#FB923C"; // orange-400
    case Types.TagColor.Orange:
      return "#EA580C"; // orange-600
    case Types.TagColor.DarkOrange:
      return "#7C2D12"; // orange-900

    case Types.TagColor.LightAmber:
      return "#FBBF24"; // amber-400
    case Types.TagColor.Amber:
      return "#D97706"; // amber-600
    case Types.TagColor.DarkAmber:
      return "#78350F"; // amber-900

    case Types.TagColor.LightYellow:
      return "#FACC15"; // yellow-400
    case Types.TagColor.Yellow:
      return "#CA8A04"; // yellow-600
    case Types.TagColor.DarkYellow:
      return "#713F12"; // yellow-900

    case Types.TagColor.LightLime:
      return "#A3E635"; // lime-400
    case Types.TagColor.Lime:
      return "#65A30D"; // lime-600
    case Types.TagColor.DarkLime:
      return "#365314"; // lime-900

    case Types.TagColor.LightGreen:
      return "#4ADE80"; // green-400
    case Types.TagColor.Green:
      return "#16A34A"; // green-600
    case Types.TagColor.DarkGreen:
      return "#14532D"; // green-900

    case Types.TagColor.LightEmerald:
      return "#34D399"; // emerald-400
    case Types.TagColor.Emerald:
      return "#059669"; // emerald-600
    case Types.TagColor.DarkEmerald:
      return "#064E3B"; // emerald-900

    case Types.TagColor.LightTeal:
      return "#2DD4BF"; // teal-400
    case Types.TagColor.Teal:
      return "#0D9488"; // teal-600
    case Types.TagColor.DarkTeal:
      return "#134E4A"; // teal-900

    case Types.TagColor.LightCyan:
      return "#22D3EE"; // cyan-400
    case Types.TagColor.Cyan:
      return "#0891B2"; // cyan-600
    case Types.TagColor.DarkCyan:
      return "#164E63"; // cyan-900

    case Types.TagColor.LightSky:
      return "#38BDF8"; // sky-400
    case Types.TagColor.Sky:
      return "#0284C7"; // sky-600
    case Types.TagColor.DarkSky:
      return "#0C4A6E"; // sky-900

    case Types.TagColor.LightBlue:
      return "#60A5FA"; // blue-400
    case Types.TagColor.Blue:
      return "#2563EB"; // blue-600
    case Types.TagColor.DarkBlue:
      return "#1E3A8A"; // blue-900

    case Types.TagColor.LightIndigo:
      return "#818CF8"; // indigo-400
    case Types.TagColor.Indigo:
      return "#4F46E5"; // indigo-600
    case Types.TagColor.DarkIndigo:
      return "#312E81"; // indigo-900

    case Types.TagColor.LightViolet:
      return "#A78BFA"; // violet-400
    case Types.TagColor.Violet:
      return "#7C3AED"; // violet-600
    case Types.TagColor.DarkViolet:
      return "#4C1D95"; // violet-900

    case Types.TagColor.LightPurple:
      return "#C084FC"; // purple-400
    case Types.TagColor.Purple:
      return "#9333EA"; // purple-600
    case Types.TagColor.DarkPurple:
      return "#581C87"; // purple-900

    case Types.TagColor.LightFuchsia:
      return "#E879F9"; // fuchsia-400
    case Types.TagColor.Fuchsia:
      return "#C026D3"; // fuchsia-600
    case Types.TagColor.DarkFuchsia:
      return "#701A75"; // fuchsia-900

    case Types.TagColor.LightPink:
      return "#F472B6"; // pink-400
    case Types.TagColor.Pink:
      return "#DB2777"; // pink-600
    case Types.TagColor.DarkPink:
      return "#831843"; // pink-900

    case Types.TagColor.LightRose:
      return "#FB7185"; // rose-400
    case Types.TagColor.Rose:
      return "#E11D48"; // rose-600
    case Types.TagColor.DarkRose:
      return "#881337"; // rose-900
  }
};

// ORIGINAL USING TAILWIND
// export const tagColor = (color?: Types.TagColor) => {
//   switch (color) {
//     case undefined:
//       return "slate-600";
//     case Types.TagColor.LightSlate:
//       return "slate-400";
//     case Types.TagColor.Slate:
//       return "slate-600";
//     case Types.TagColor.DarkSlate:
//       return "slate-900";

//     case Types.TagColor.LightRed:
//       return "red-400";
//     case Types.TagColor.Red:
//       return "red-600";
//     case Types.TagColor.DarkRed:
//       return "red-900";

//     case Types.TagColor.LightOrange:
//       return "orange-400";
//     case Types.TagColor.Orange:
//       return "orange-600";
//     case Types.TagColor.DarkOrange:
//       return "orange-900";

//     case Types.TagColor.LightAmber:
//       return "amber-400";
//     case Types.TagColor.Amber:
//       return "amber-600";
//     case Types.TagColor.DarkAmber:
//       return "amber-900";

//     case Types.TagColor.LightYellow:
//       return "yellow-400";
//     case Types.TagColor.Yellow:
//       return "yellow-600";
//     case Types.TagColor.DarkYellow:
//       return "yellow-900";

//     case Types.TagColor.LightLime:
//       return "lime-400";
//     case Types.TagColor.Lime:
//       return "lime-600";
//     case Types.TagColor.DarkLime:
//       return "lime-900";

//     case Types.TagColor.LightGreen:
//       return "green-400";
//     case Types.TagColor.Green:
//       return "green-600";
//     case Types.TagColor.DarkGreen:
//       return "green-900";

//     case Types.TagColor.LightEmerald:
//       return "emerald-400";
//     case Types.TagColor.Emerald:
//       return "emerald-600";
//     case Types.TagColor.DarkEmerald:
//       return "emerald-900";

//     case Types.TagColor.LightTeal:
//       return "teal-400";
//     case Types.TagColor.Teal:
//       return "teal-600";
//     case Types.TagColor.DarkTeal:
//       return "teal-900";

//     case Types.TagColor.LightCyan:
//       return "cyan-400";
//     case Types.TagColor.Cyan:
//       return "cyan-600";
//     case Types.TagColor.DarkCyan:
//       return "cyan-900";

//     case Types.TagColor.LightSky:
//       return "sky-400";
//     case Types.TagColor.Sky:
//       return "sky-600";
//     case Types.TagColor.DarkSky:
//       return "sky-900";

//     case Types.TagColor.LightBlue:
//       return "blue-400";
//     case Types.TagColor.Blue:
//       return "blue-600";
//     case Types.TagColor.DarkBlue:
//       return "blue-900";

//     case Types.TagColor.LightIndigo:
//       return "indigo-400";
//     case Types.TagColor.Indigo:
//       return "indigo-600";
//     case Types.TagColor.DarkIndigo:
//       return "indigo-900";

//     case Types.TagColor.LightViolet:
//       return "violet-400";
//     case Types.TagColor.Violet:
//       return "violet-600";
//     case Types.TagColor.DarkViolet:
//       return "violet-900";

//     case Types.TagColor.LightPurple:
//       return "purple-400";
//     case Types.TagColor.Purple:
//       return "purple-600";
//     case Types.TagColor.DarkPurple:
//       return "purple-900";

//     case Types.TagColor.LightFuchsia:
//       return "fuchsia-400";
//     case Types.TagColor.Fuchsia:
//       return "fuchsia-600";
//     case Types.TagColor.DarkFuchsia:
//       return "fuchsia-900";

//     case Types.TagColor.LightPink:
//       return "pink-400";
//     case Types.TagColor.Pink:
//       return "pink-600";
//     case Types.TagColor.DarkPink:
//       return "pink-900";

//     case Types.TagColor.LightRose:
//       return "rose-400";
//     case Types.TagColor.Rose:
//       return "rose-600";
//     case Types.TagColor.DarkRose:
//       return "rose-900";
//   }
// };
