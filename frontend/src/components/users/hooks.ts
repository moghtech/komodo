import { useRead } from "@lib/hooks";
import { Types } from "komodo_client";
import { UsableResource } from "@types";

export const useUserTargetPermissions = (user_target: Types.UserTarget) => {
  const permissions = useRead("ListUserTargetPermissions", {
    user_target,
  }).data;
  const servers = useRead("ListServers", {}).data;
  const stacks = useRead("ListStacks", {}).data;
  const deployments = useRead("ListDeployments", {}).data;
  const builds = useRead("ListBuilds", {}).data;
  const repos = useRead("ListRepos", {}).data;
  const procedures = useRead("ListProcedures", {}).data;
  const builders = useRead("ListBuilders", {}).data;
  const alerters = useRead("ListAlerters", {}).data;
  const syncs = useRead("ListResourceSyncs", {}).data;
  const perms: (Types.Permission & { name: string })[] = [];
  addPerms(user_target, permissions, "Server", servers, perms);
  addPerms(user_target, permissions, "Stack", stacks, perms);
  addPerms(user_target, permissions, "Deployment", deployments, perms);
  addPerms(user_target, permissions, "Build", builds, perms);
  addPerms(user_target, permissions, "Repo", repos, perms);
  addPerms(user_target, permissions, "Procedure", procedures, perms);
  addPerms(user_target, permissions, "Builder", builders, perms);
  addPerms(user_target, permissions, "Alerter", alerters, perms);
  addPerms(user_target, permissions, "ResourceSync", syncs, perms);
  return perms;
};

function addPerms<I>(
  user_target: Types.UserTarget,
  permissions: Types.Permission[] | undefined,
  resource_type: UsableResource,
  resources: Types.ResourceListItem<I>[] | undefined,
  perms: (Types.Permission & { name: string })[]
) {
  resources?.forEach((resource) => {
    const perm = permissions?.find(
      (p) =>
        p.resource_target.type === resource_type &&
        p.resource_target.id === resource.id
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
