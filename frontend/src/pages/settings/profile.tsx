import { ConfirmButton, CopyButton } from "@components/util";
import {
  useInvalidate,
  useManageUser,
  useRead,
  useSetTitle,
  useUser,
  useWrite,
} from "@lib/hooks";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@ui/dialog";
import { Button } from "@ui/button";
import { useToast } from "@ui/use-toast";
import {
  Trash,
  PlusCircle,
  Loader2,
  Check,
  User,
  Eye,
  EyeOff,
  KeyRound,
  UserPen,
} from "lucide-react";
import { useState } from "react";
import { Input } from "@ui/input";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@ui/dropdown-menu";
import { ApiKeysTable } from "@components/api-keys/table";
import { Section } from "@components/layouts";
import { Card, CardHeader } from "@ui/card";
import { Types } from "komodo_client";

export const Profile = () => {
  useSetTitle("Profile");
  const user = useUser().data;
  if (!user) {
    return (
      <div className="w-full h-[400px] flex justify-center items-center">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }
  return <ProfileInner user={user} />;
};

const ProfileInner = ({ user }: { user: Types.User }) => {
  const { refetch: refetchUser } = useUser();
  const { toast } = useToast();
  const keys = useRead("ListApiKeys", {}).data ?? [];
  const [username, setUsername] = useState(user.username);
  const [password, setPassword] = useState("");
  const [hidePassword, setHidePassword] = useState(true);
  const { mutate: updateUsername } = useWrite("UpdateUserUsername", {
    onSuccess: () => {
      toast({ title: "Username updated." });
      refetchUser();
    },
  });
  const { mutate: updatePassword } = useWrite("UpdateUserPassword", {
    onSuccess: () => {
      toast({ title: "Password updated." });
      setPassword("");
    },
  });
  return (
    <div className="flex flex-col gap-6">
      {/* Profile */}
      <Section title="Profile" icon={<User className="w-4 h-4" />}>
        <Card>
          <CardHeader className="gap-4">
            {/* Profile Info */}
            <UserProfile user={user} />

            {/* Update Username */}
            <div className="flex items-center gap-4">
              <div className="text-muted-foreground font-mono">Username:</div>
              <div className="w-[200px] lg:w-[300px]">
                <Input
                  placeholder="Input username"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                />
              </div>
              <ConfirmButton
                title="Update Username"
                icon={<UserPen className="w-4 h-4" />}
                onClick={() => updateUsername({ username })}
                disabled={!username || username === user.username}
              />
            </div>

            {/* Update Password */}
            {user.config.type === "Local" && (
              <div className="flex items-center gap-4">
                <div className="text-muted-foreground font-mono">Password:</div>
                <div className="w-[200px] lg:w-[300px] flex items-center gap-2">
                  <Input
                    placeholder="Input password"
                    type={hidePassword ? "password" : "text"}
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                  />
                  <Button
                    size="icon"
                    variant="outline"
                    onClick={() => setHidePassword((curr) => !curr)}
                  >
                    {hidePassword ? (
                      <EyeOff className="w-4 h-4" />
                    ) : (
                      <Eye className="w-4 h-4" />
                    )}
                  </Button>
                </div>
                <ConfirmButton
                  title="Update Password"
                  icon={<UserPen className="w-4 h-4" />}
                  onClick={() => updatePassword({ password })}
                  disabled={!password}
                />
              </div>
            )}
          </CardHeader>
        </Card>
      </Section>

      {/* Api Keys */}
      <Section title="Api Keys" icon={<KeyRound className="w-4 h-4" />}>
        <div>
          <CreateKey />
        </div>
        <ApiKeysTable keys={keys} DeleteKey={DeleteKey} />
      </Section>
    </div>
  );
};

const UserProfile = ({ user }: { user: Types.User }) => {
  return (
    <div className="flex items-center gap-4 flex-wrap">
      <div className="font-mono text-muted-foreground">Type:</div>
      {user.config.type}

      <div className="font-mono text-muted-foreground">|</div>

      <div className="font-mono text-muted-foreground">Admin:</div>
      {user.admin ? "True" : "False"}

      {user.admin && (
        <>
          <div className="font-mono text-muted-foreground">|</div>

          <div className="font-mono text-muted-foreground">Super Admin:</div>
          {user.super_admin ? "True" : "False"}
        </>
      )}
    </div>
  );
};

const ONE_DAY_MS = 1000 * 60 * 60 * 24;

type ExpiresOptions = "90 days" | "180 days" | "1 year" | "never";

const CreateKey = () => {
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [expires, setExpires] = useState<ExpiresOptions>("never");
  const [submitted, setSubmitted] = useState<{ key: string; secret: string }>();
  const invalidate = useInvalidate();
  const { mutate, isPending } = useManageUser("CreateApiKey", {
    onSuccess: ({ key, secret }) => {
      invalidate(["ListApiKeys"]);
      setSubmitted({ key, secret });
    },
  });
  const now = Date.now();
  const expiresOptions: Record<ExpiresOptions, number> = {
    "90 days": now + ONE_DAY_MS * 90,
    "180 days": now + ONE_DAY_MS * 180,
    "1 year": now + ONE_DAY_MS * 365,
    never: 0,
  };
  const submit = () => mutate({ name, expires: expiresOptions[expires] });
  const onOpenChange = (open: boolean) => {
    setOpen(open);
    if (!open) {
      setName("");
      setExpires("never");
      setSubmitted(undefined);
    }
  };
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogTrigger asChild>
        <Button variant="secondary" className="items-center gap-2">
          New Api Key <PlusCircle className="w-4 h-4" />
        </Button>
      </DialogTrigger>
      <DialogContent>
        {submitted ? (
          <>
            <DialogHeader>
              <DialogTitle>Api Key Created</DialogTitle>
            </DialogHeader>
            <div className="py-8 flex flex-col gap-4">
              <div className="flex items-center justify-between">
                Key
                <Input className="w-72" value={submitted.key} disabled />
                <CopyButton content={submitted.key} />
              </div>
              <div className="flex items-center justify-between">
                Secret
                <Input className="w-72" value={submitted.secret} disabled />
                <CopyButton content={submitted.secret} />
              </div>
            </div>
            <DialogFooter className="flex justify-end">
              <Button
                variant="secondary"
                className="gap-4"
                onClick={() => onOpenChange(false)}
              >
                Confirm <Check className="w-4" />
              </Button>
            </DialogFooter>
          </>
        ) : (
          <>
            <DialogHeader>
              <DialogTitle>Create Api Key</DialogTitle>
            </DialogHeader>
            <div className="py-8 flex flex-col gap-4">
              <div className="flex items-center justify-between">
                Name
                <Input
                  className="w-72"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                />
              </div>
              <div className="flex items-center justify-between">
                Expiry
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button
                      className="w-36 justify-between px-3"
                      variant="outline"
                    >
                      {expires}
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent className="w-36" side="bottom">
                    <DropdownMenuGroup>
                      {Object.keys(expiresOptions)
                        .filter((option) => option !== expires)
                        .map((option) => (
                          <DropdownMenuItem
                            key={option}
                            onClick={() => setExpires(option as any)}
                          >
                            {option}
                          </DropdownMenuItem>
                        ))}
                    </DropdownMenuGroup>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            </div>
            <DialogFooter className="flex justify-end">
              <Button
                variant="secondary"
                className="gap-4"
                onClick={submit}
                disabled={isPending}
              >
                Submit
                {isPending ? (
                  <Loader2 className="w-4 animate-spin" />
                ) : (
                  <Check className="w-4" />
                )}
              </Button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
};

const DeleteKey = ({ api_key }: { api_key: string }) => {
  const invalidate = useInvalidate();
  const { toast } = useToast();
  const { mutate, isPending } = useManageUser("DeleteApiKey", {
    onSuccess: () => {
      invalidate(["ListApiKeys"]);
      toast({ title: "Api Key Deleted" });
    },
    onError: () => {
      toast({ title: "Failed to delete api key", variant: "destructive" });
    },
  });
  return (
    <ConfirmButton
      title="Delete"
      variant="destructive"
      icon={<Trash className="w-4 h-4" />}
      onClick={(e) => {
        e.stopPropagation();
        mutate({ key: api_key });
      }}
      loading={isPending}
    />
  );
};
