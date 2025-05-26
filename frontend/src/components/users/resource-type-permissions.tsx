import { PermissionLevelSelector } from "@components/config/util";
import { useInvalidate, useRead, useWrite } from "@lib/hooks";
import { RESOURCE_TARGETS } from "@lib/utils";
import { Types } from "komodo_client";
import { useToast } from "@ui/use-toast";
import { Card, CardContent, CardHeader } from "@ui/card";

export const UserTargetPermissionsOnResourceTypes = ({
  user_target,
}: {
  user_target: Types.UserTarget;
}) => {
  const { toast } = useToast();
  const inv = useInvalidate();

  const { mutate } = useWrite("UpdatePermissionOnResourceType", {
    onSuccess: () => {
      toast({ title: "Updated permissions on target" });
      if (user_target.type === "User") {
        inv(["FindUser", { user: user_target.id }]);
      } else if (user_target.type === "UserGroup") {
        inv(["GetUserGroup", { user_group: user_target.id }]);
      }
    },
  });

  const update = (resource_type, permission) =>
    mutate({ user_target, resource_type, permission });

  if (user_target.type === "User") {
    return (
      <UserPermissionsOnResourceType user_id={user_target.id} update={update} />
    );
  } else if (user_target.type === "UserGroup") {
    return (
      <UserGroupPermissionsOnResourceType
        group_id={user_target.id}
        update={update}
      />
    );
  }
};

const UserPermissionsOnResourceType = ({
  user_id,
  update,
}: {
  user_id: string;
  update: (
    resource_type: Types.ResourceTarget["type"],
    permission: Types.PermissionLevel
  ) => void;
}) => {
  const user = useRead("FindUser", { user: user_id }).data;
  return <PermissionsOnResourceType all={user?.all} update={update} />;
};

const UserGroupPermissionsOnResourceType = ({
  group_id,
  update,
}: {
  group_id: string;
  update: (
    resource_type: Types.ResourceTarget["type"],
    permission: Types.PermissionLevel
  ) => void;
}) => {
  const group = useRead("GetUserGroup", { user_group: group_id }).data;
  return <PermissionsOnResourceType all={group?.all} update={update} />;
};

const PermissionsOnResourceType = ({
  all,
  update,
}: {
  all: Types.User["all"];
  update: (
    resource_type: Types.ResourceTarget["type"],
    permission: Types.PermissionLevel
  ) => void;
}) => {
  return (
    <Card>
      <CardHeader className="border-b pb-6">Base Permissions</CardHeader>
      <CardContent className="mt-6 grid gap-4 grid-cols-1 md:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4">
        {RESOURCE_TARGETS.map((type) => {
          const permission = all?.[type] ?? Types.PermissionLevel.None;
          return (
            <div
              key={type}
              className="flex items-center justify-between w-[270px]"
            >
              {type}:
              <PermissionLevelSelector
                level={
                  typeof permission === "string" ? permission : permission.level
                }
                onSelect={(level) => update(type, level)}
              />
            </div>
          );
        })}
      </CardContent>
    </Card>
  );
};
