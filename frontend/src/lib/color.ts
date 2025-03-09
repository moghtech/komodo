import { Types } from "komodo_client";

export type ColorIntention =
  | "Good"
  | "Neutral"
  | "Warning"
  | "Critical"
  | "Unknown"
  | "None";

export const hex_color_by_intention = (intention: ColorIntention) => {
  switch (intention) {
    case "Good":
      return "#22C55E";
    case "Neutral":
      return "#3B82F6";
    case "Warning":
      return "#F97316";
    case "Critical":
      return "#EF0044";
    case "Unknown":
      return "#A855F7";
    case "None":
      return "";
  }
};

export const fill_color_class_by_intention = (intention: ColorIntention) => {
  switch (intention) {
    case "Good":
      return "text-green-400 dark:text-green-700";
    case "Neutral":
      return "text-blue-400 dark:text-blue-700";
    case "Warning":
      return "text-orange-400 dark:text-orange-700";
    case "Critical":
      return "text-red-400 dark:text-red-700";
    case "Unknown":
      return "text-purple-400 dark:text-purple-700";
    case "None":
      return "";
  }
};

export const stroke_color_class_by_intention = (intention: ColorIntention) => {
  switch (intention) {
    case "Good":
      return "stroke-green-600 dark:stroke-green-500";
    case "Neutral":
      return "stroke-blue-600 dark:stroke-blue-500";
    case "Warning":
      return "stroke-orange-600 dark:stroke-orange-500";
    case "Critical":
      return "stroke-red-600 dark:stroke-red-500";
    case "Unknown":
      return "stroke-purple-600 dark:stroke-purple-500";
    case "None":
      return "";
  }
};

export const bg_color_class_by_intention = (intention: ColorIntention) => {
  switch (intention) {
    case "Good":
      return "bg-green-400 dark:bg-green-700";
    case "Neutral":
      return "bg-blue-400 dark:bg-blue-700";
    case "Warning":
      return "bg-orange-400 dark:bg-orange-500";
    case "Critical":
      return "bg-red-400 dark:bg-red-700";
    case "Unknown":
      return "bg-purple-400 dark:bg-purple-700";
    case "None":
      return "";
  }
};

export const border_color_class_by_intention = (intention: ColorIntention) => {
  switch (intention) {
    case "Good":
      return "border-green-700 dark:border-green-400";
    case "Neutral":
      return "border-blue-700 dark:border-blue-400";
    case "Warning":
      return "border-orange-700 dark:border-orange-400";
    case "Critical":
      return "border-red-700 dark:border-red-400";
    case "Unknown":
      return "border-purple-700 dark:border-purple-400";
    case "None":
      return "";
  }
};

export const text_color_class_by_intention = (intention: ColorIntention) => {
  switch (intention) {
    case "Good":
      return "text-green-700 dark:text-green-400";
    case "Neutral":
      return "text-blue-700 dark:text-blue-400";
    case "Warning":
      return "text-orange-700 dark:text-orange-400";
    case "Critical":
      return "text-red-700 dark:text-red-400";
    case "Unknown":
      return "text-purple-700 dark:text-purple-400";
    case "None":
      return "";
  }
};

export const soft_text_color_class_by_intention = (
  intention: ColorIntention
) => {
  switch (intention) {
    case "Good":
      return "text-green-700/60 dark:text-green-400/60";
    case "Neutral":
      return "text-blue-700/60 dark:text-blue-400/60";
    case "Warning":
      return "text-orange-700/60 dark:text-orange-400/60";
    case "Critical":
      return "text-red-700/60 dark:text-red-400/60";
    case "Unknown":
      return "text-purple-700/60 dark:text-purple-400/60";
    case "None":
      return "";
  }
};

export const server_state_intention: (
  status?: Types.ServerState
) => ColorIntention = (status) => {
  switch (status) {
    case Types.ServerState.Ok:
      return "Good";
    case Types.ServerState.NotOk:
      return "Critical";
    case Types.ServerState.Disabled:
      return "Neutral";
    case undefined:
      return "None";
  }
};

export const deployment_state_intention: (
  state?: Types.DeploymentState
) => ColorIntention = (state) => {
  switch (state) {
    case undefined:
      return "None";
    case Types.DeploymentState.Running:
      return "Good";
    case Types.DeploymentState.NotDeployed:
      return "Neutral";
    case Types.DeploymentState.Paused:
      return "Warning";
    case Types.DeploymentState.Unknown:
      return "Unknown";
    default:
      return "Critical";
  }
};

export const container_state_intention: (
  state?: Types.ContainerStateStatusEnum
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

export const build_state_intention = (status?: Types.BuildState) => {
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

export const repo_state_intention = (state?: Types.RepoState) => {
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

export const stack_state_intention = (state?: Types.StackState) => {
  switch (state) {
    case undefined:
      return "None";
    case Types.StackState.Running:
      return "Good";
    case Types.StackState.Paused:
      return "Warning";
    case Types.StackState.Stopped:
      return "Critical";
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

export const procedure_state_intention = (status?: Types.ProcedureState) => {
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

export const action_state_intention = (status?: Types.ActionState) => {
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

export const resource_sync_state_intention = (
  status?: Types.ResourceSyncState
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

export const alert_level_intention: (
  level: Types.SeverityLevel
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

export const diff_type_intention: (
  level: Types.DiffData["type"],
  reverse: boolean
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
