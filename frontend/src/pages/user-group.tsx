import { ExportButton } from "@components/export";
import { Page, Section } from "@components/layouts";
import { PermissionsTableTabs } from "@components/users/permissions-table";
import { UserTable } from "@components/users/table";
import { ActionWithDialog } from "@components/util";
import { useInvalidate, useRead, useWrite } from "@lib/hooks";
import { filterBySplit } from "@lib/utils";
import { Types } from "komodo_client";
import { Button } from "@ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@ui/command";
import { Input } from "@ui/input";
import { Popover, PopoverContent, PopoverTrigger } from "@ui/popover";
import { useToast } from "@ui/use-toast";
import { PlusCircle, Save, SearchX, Trash, User, Users } from "lucide-react";
import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";

export const UserGroupPage = () => {
  const { toast } = useToast();
  const inv = useInvalidate();
  const group_id = useParams().id as string;
  const group = useRead("ListUserGroups", {}).data?.find(
    (group) => group._id?.$oid === group_id
  );
  const users = useRead("ListUsers", {}).data;
  const [name, setName] = useState("");
  const renameMutate = useWrite("RenameUserGroup", {
    onSuccess: () => {
      inv(["ListUserGroups"]);
      toast({ title: "Renamed User Group" });
    },
  }).mutate;
  const rename = () => {
    if (!name) {
      toast({ title: "New name cannot be empty" });
      if (group) setName(group.name);
      return;
    }
    renameMutate({ id: group_id, name });
  };
  const removeMutate = useWrite("RemoveUserFromUserGroup", {
    onSuccess: () => {
      inv(["ListUserGroups"]);
      toast({ title: "Removed User from User Group" });
    },
  }).mutate;
  if (!group) return null;
  return (
    <Page
      title={group.name}
      icon={<Users className="w-8 h-8" />}
      actions={<ExportButton user_groups={[group_id]} />}
      subtitle={
        <div className="text-sm text-muted-foreground flex gap-2">
          <div>User Group</div>|
          {(group.users ?? []).length > 0 && (
            <div>
              {(group.users ?? []).length} User
              {(group.users ?? []).length > 1 ? "s" : ""}
            </div>
          )}
          {(group.users ?? []).length === 0 && <div>No Users</div>}
        </div>
      }
    >
      <Section
        title="Users"
        icon={<User className="w-4 h-4" />}
        actions={<AddUserToGroup group_id={group_id} />}
      >
        <UserTable
          users={
            users?.filter((user) =>
              group ? (group.users ?? []).includes(user._id?.$oid!) : false
            ) ?? []
          }
          onUserRemove={(user_id) =>
            removeMutate({ user_group: group_id, user: user_id })
          }
        />
      </Section>

      <PermissionsTableTabs user_target={{ type: "UserGroup", id: group_id }} />

      <div className="flex flex-col justify-end w-full gap-4">
        <div className="flex justify-end w-full">
          <div className="flex items-center gap-2">
            <h2 className="text-muted-foreground">Rename</h2>
            <Input
              placeholder="Enter new name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") rename();
              }}
              className="w-[300px]"
            />
            <Button variant="secondary" onClick={rename}>
              <Save className="w-4 h-4" />
            </Button>
          </div>
        </div>
        <div className="flex justify-end w-full">
          <DeleteUserGroup group={group} />
        </div>
      </div>
    </Page>
  );
};

const AddUserToGroup = ({ group_id }: { group_id: string }) => {
  const inv = useInvalidate();
  const { toast } = useToast();

  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");

  const group = useRead("ListUserGroups", {}).data?.find(
    (group) => group._id?.$oid === group_id
  );

  const users = useRead("ListUsers", {}).data?.filter(
    (user) =>
      // Only show users not already in group
      !group?.users?.includes(user._id?.$oid!)
  );

  const addUser = useWrite("AddUserToUserGroup", {
    onSuccess: () => {
      inv(["ListUserGroups"]);
      toast({ title: "Added User to User Group" });
    },
  }).mutate;

  if (!users || users.length === 0) return null;

  const filtered = filterBySplit(users, search, (item) => item.username);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="secondary"
          className="flex justify-start items-center gap-2 w-48 px-3"
        >
          <PlusCircle className="w-4 h-4" />
          Add User
        </Button>
      </PopoverTrigger>
      <PopoverContent
        className="w-[300px] max-h-[400px] p-0"
        sideOffset={12}
        align="end"
      >
        <Command shouldFilter={false}>
          <CommandInput
            placeholder="Search Users"
            className="h-9"
            value={search}
            onValueChange={setSearch}
          />
          <CommandList>
            <CommandEmpty className="flex justify-evenly items-center">
              No Users Found
              <SearchX className="w-3 h-3" />
            </CommandEmpty>

            <CommandGroup>
              {filtered?.map((user) => (
                <CommandItem
                  key={user.username}
                  onSelect={() => {
                    setOpen(false);
                    addUser({ user_group: group_id, user: user._id?.$oid! });
                  }}
                >
                  <Button
                    variant="ghost"
                    className="flex gap-2 items-center p-0"
                  >
                    <UserAvatar avatar={(user.config.data as any).avatar} />
                    {user.username}
                  </Button>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
};

const UserAvatar = ({ avatar }: { avatar: string | undefined }) =>
  avatar ? (
    <img src={avatar} alt="Avatar" className="w-4" />
  ) : (
    <User className="w-4" />
  );

export const DeleteUserGroup = ({ group }: { group: Types.UserGroup }) => {
  const nav = useNavigate();
  const inv = useInvalidate();
  const { toast } = useToast();
  const { mutate, isPending } = useWrite("DeleteUserGroup", {
    onSuccess: () => {
      inv(
        ["ListUserGroups"],
        ["GetUserGroup", { user_group: group._id?.$oid! }]
      );
      toast({ title: `Deleted User Group ${group.name}` });
      nav("/settings");
    },
  });

  return (
    <ActionWithDialog
      name={group.name}
      title="Delete"
      icon={<Trash className="h-4 w-4" />}
      variant="destructive"
      onClick={() => mutate({ id: group._id?.$oid! })}
      disabled={isPending}
      loading={isPending}
    />
  );
};
