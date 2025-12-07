import { ApiKeysTable as InnerApiKeysTable } from "@components/api-keys/table";
import { Page } from "@components/layouts";
import { PermissionsTableTabs } from "@components/users/permissions-table";
import {
  CreateKeyForServiceUser,
  DeleteKeyForServiceUser,
} from "@components/users/service-api-key";
import { ConfirmButton } from "@components/util";
import { useInvalidate, useRead, useUser, useWrite } from "@lib/hooks";
import { Label } from "@ui/label";
import { Switch } from "@ui/switch";
import { useToast } from "@ui/use-toast";
import { UserCheck, UserMinus, Users } from "lucide-react";
import { Link, useParams } from "react-router-dom";
import { Button } from "@ui/button";
import { Card, CardContent, CardHeader } from "@ui/card";

export default function UserPage() {
  const admin_user = useUser().data;
  const { toast } = useToast();
  const inv = useInvalidate();
  const user_id = useParams().id as string;
  const user = useRead("ListUsers", {}).data?.find(
    (user) => user._id?.$oid === user_id
  );
  const { mutate: update_base } = useWrite("UpdateUserBasePermissions", {
    onSuccess: () => {
      inv(["FindUser"]);
      inv(["ListUsers"]);
      toast({ title: "Modify user base permissions" });
    },
  });
  const { mutate: update_admin } = useWrite("UpdateUserAdmin", {
    onSuccess: () => {
      inv(["FindUser"]);
      inv(["ListUsers"]);
      toast({ title: "Modify user admin" });
    },
  });
  const enabledClass = user?.enabled ? "text-green-500" : "text-red-500";
  const avatar = (user?.config.data as any)?.avatar as string | undefined;
  if (!user || !admin_user) return null;
  return (
    <Page
      title={user?.username}
      icon={avatar && <img src={avatar} alt="" className="w-7 h-7" />}
      subtitle={
        <div className="text-sm text-muted-foreground flex gap-2">
          <div className={enabledClass}>
            {user?.enabled ? "Enabled" : "Disabled"}
          </div>
          |
          <div className="flex gap-2">
            Level:{" "}
            <div className="font-bold">{user?.admin ? "Admin" : "User"}</div>
          </div>
          |
          <div className="flex gap-2">
            Type: <div className="font-bold">{user?.config.type}</div>
          </div>
        </div>
      }
    >
      {(!user.admin || (!user.super_admin && admin_user.super_admin)) && (
        <Card>
          <CardHeader className="border-b pb-6">User Permissions</CardHeader>
          <CardContent className="mt-6 flex gap-8 items-center flex-wrap">
            <ConfirmButton
              title={user.enabled ? "Disable User" : "Enable User"}
              icon={
                user.enabled ? (
                  <UserMinus className="w-4 h-4" />
                ) : (
                  <UserCheck className="w-4 h-4" />
                )
              }
              variant={user.enabled ? "destructive" : "outline"}
              onClick={() => update_base({ user_id, enabled: !user.enabled })}
            />
            <ConfirmButton
              title={user.admin ? "Take Admin" : "Make Admin"}
              icon={
                user.admin ? (
                  <UserMinus className="w-4 h-4" />
                ) : (
                  <UserCheck className="w-4 h-4" />
                )
              }
              variant={user.admin ? "destructive" : "outline"}
              onClick={() => update_admin({ user_id, admin: !user.admin })}
            />
            {user.enabled &&
              !user.admin &&
              (["Server", "Build"] as Array<"Server" | "Build">).map((item) => {
                const key = `create_${item.toLowerCase()}_permissions` as
                  | "create_server_permissions"
                  | "create_build_permissions";
                const req_key = `create_${item.toLowerCase()}s`;
                return (
                  <div
                    className="flex items-center gap-4 cursor-pointer p-2"
                    onClick={() =>
                      update_base({ user_id, [req_key]: !user[key] })
                    }
                  >
                    <Label className="cursor-pointer" htmlFor={key}>
                      Create {item} Permission
                    </Label>
                    <Switch
                      id={key}
                      className="flex gap-4"
                      checked={user[key]}
                    />
                  </div>
                );
              })}
          </CardContent>
        </Card>
      )}
      {user.config.type === "Service" && <ApiKeysTable user_id={user_id} />}
      {user.enabled && !user.admin && (
        <>
          <Groups user_id={user_id} />
          <PermissionsTableTabs
            user_target={{ type: "User", id: user._id?.$oid! }}
          />
        </>
      )}
    </Page>
  );
}

const ApiKeysTable = ({ user_id }: { user_id: string }) => {
  const keys = useRead("ListApiKeysForServiceUser", { user: user_id }).data;
  return (
    <Card>
      <CardHeader className="border-b pb-6 flex flex-row items-center gap-4">
        Api Keys <CreateKeyForServiceUser user_id={user_id} />
      </CardHeader>
      <CardContent>
        <InnerApiKeysTable
          keys={keys ?? []}
          DeleteKey={DeleteKeyForServiceUser}
        />
      </CardContent>
    </Card>
  );
};

const Groups = ({ user_id }: { user_id: string }) => {
  const groups = useRead("ListUserGroups", {}).data?.filter((group) =>
    group.users?.includes(user_id)
  );
  if (!groups || groups.length === 0) {
    return null;
  }
  return (
    <Card>
      <CardHeader className="border-b pb-6">Groups</CardHeader>
      <CardContent className="mt-6 grid gap-4 grid-cols-1 md:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4">
        {groups.map((group) => (
          <Link to={`/user-groups/${group._id?.$oid}`}>
            <Button variant="link" className="flex gap-2 items-center p-0">
              <Users className="w-4 h-4" />
              {group.name}
            </Button>
          </Link>
        ))}
      </CardContent>
    </Card>
  );
};
