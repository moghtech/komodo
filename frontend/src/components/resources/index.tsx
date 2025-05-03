import { RequiredResourceComponents, UsableResource } from "@types";
import { AlerterComponents } from "./alerter";
import { BuildComponents } from "./build";
import { BuilderComponents } from "./builder";
import { DeploymentComponents } from "./deployment";
import { RepoComponents } from "./repo";
import { ServerComponents } from "./server";
import { ProcedureComponents } from "./procedure/index";
import { ResourceSyncComponents } from "./resource-sync";
import { StackComponents } from "./stack";
import { ActionComponents } from "./action";

export const ResourceComponents: {
  [key in UsableResource]: RequiredResourceComponents;
} = {
  Server: ServerComponents,
  Stack: StackComponents,
  Deployment: DeploymentComponents,
  Build: BuildComponents,
  Repo: RepoComponents,
  Procedure: ProcedureComponents,
  Action: ActionComponents,
  ResourceSync: ResourceSyncComponents,
  Builder: BuilderComponents,
  Alerter: AlerterComponents,
};
