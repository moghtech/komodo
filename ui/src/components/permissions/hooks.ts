import { useAllResources, useDebouncedTermSearch, useRead } from "@/lib/hooks";
import { UsableResource } from "@/resources";
import { Types } from "komodo_client";
import { useMemo } from "react";

export function useUserTargetPermissions(userTarget: Types.UserTarget) {
  const { search, setSearch, terms } = useDebouncedTermSearch();

  const allPermissions = useRead("ListUserTargetPermissions", {
    user_target: userTarget,
  }).data;

  // Can't limit without query taking into account
  // specific user target access
  const allResources = useAllResources(terms, 0);

  const permissions = useMemo(() => {
    const permissions: (Types.Permission & { name: string })[] = [];
    for (const [resourceType, resources] of Object.entries(allResources)) {
      addUserTargetPermissions(
        userTarget,
        allPermissions,
        resourceType as UsableResource,
        resources,
        permissions,
      );
    }
    return permissions;
  }, [
    userTarget,
    allPermissions,
    // Diff against resource arrays individually
    ...Object.values(allResources),
  ]);

  return {
    permissions,
    search,
    setSearch,
  };
}

function addUserTargetPermissions<I>(
  userTarget: Types.UserTarget,
  allPermissions: Types.Permission[] | undefined,
  resourceType: UsableResource,
  resources: Types.ResourceListItem<I>[] | undefined,
  permissions: (Types.Permission & { name: string })[],
) {
  resources?.forEach((resource) => {
    const perm = allPermissions?.find(
      (p) =>
        p.resource_target.type === resourceType &&
        p.resource_target.id === resource.id,
    );
    if (perm) {
      permissions.push({ ...perm, name: resource.name });
    } else {
      permissions.push({
        user_target: userTarget,
        name: resource.name,
        level: Types.PermissionLevel.None,
        resource_target: { type: resourceType, id: resource.id },
      });
    }
  });
}
