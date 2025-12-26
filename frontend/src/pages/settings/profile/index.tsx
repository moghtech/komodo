import { ConfirmButton, StatusBadge } from "@components/util";
import {
  useLoginOptions,
  useManageUser,
  useRead,
  useSetTitle,
  useUser,
} from "@lib/hooks";
import { Button } from "@ui/button";
import { useToast } from "@ui/use-toast";
import {
  Loader2,
  User,
  Eye,
  EyeOff,
  KeyRound,
  UserPen,
  Trash,
  CirclePlus,
} from "lucide-react";
import { useMemo, useState } from "react";
import { Input } from "@ui/input";
import { ApiKeysTable } from "@components/api-keys/table";
import { Section } from "@components/layouts";
import { Card, CardHeader } from "@ui/card";
import { Types } from "komodo_client";
import { CreateKey, DeleteKey } from "./api-key";
import { EnrollTotp } from "./totp";
import { EnrollPasskey } from "./passkey";
import { KOMODO_BASE_URL } from "@main";
import { DataTable } from "@ui/data-table";
import { Switch } from "@ui/switch";
import { Label } from "@ui/label";
import { cn } from "@lib/utils";

type LinkedLoginProvider = "Local" | "Github" | "Google" | "Oidc";

const useLinkWithOauth = () => {
  const { mutateAsync } = useManageUser("BeginThirdPartyLoginLink");
  return (provider: LinkedLoginProvider) =>
    mutateAsync({}).then(() =>
      location.replace(`${KOMODO_BASE_URL}/auth/${provider.toLowerCase()}/link`)
    );
};

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
  const options = useLoginOptions().data;
  const [username, setUsername] = useState(user.username);
  const [password, setPassword] = useState("");
  const [hidePassword, setHidePassword] = useState(true);
  const { mutate: updateUsername } = useManageUser("UpdateUsername", {
    onSuccess: () => {
      toast({ title: "Username updated." });
      refetchUser();
    },
  });
  const { mutate: updatePassword } = useManageUser("UpdatePassword", {
    onSuccess: () => {
      toast({ title: "Password updated." });
      setPassword("");
      refetchUser();
    },
  });
  const { mutate: update_skip_2fa } = useManageUser("UpdateThirdPartySkip2fa", {
    onSuccess: () => {
      toast({ title: "Updated user third party skip 2fa." });
      refetchUser();
    },
  });
  const { mutate: unlink } = useManageUser("UnlinkLogin", {
    onSuccess: () => {
      toast({ title: "Unlinked login." });
      refetchUser();
    },
  });
  const thirdPartyProviders: Array<{
    provider: LinkedLoginProvider;
    enabled: boolean;
    linked: boolean;
  }> = useMemo(
    () =>
      [
        {
          provider: "Local" as LinkedLoginProvider,
          enabled: !!options?.local,
          linked: !!user?.linked_logins?.Local,
        },
        {
          provider: "Google" as LinkedLoginProvider,
          enabled: !!options?.google,
          linked: !!user?.linked_logins?.Google,
        },
        {
          provider: "Github" as LinkedLoginProvider,
          enabled: !!options?.github,
          linked: !!user?.linked_logins?.Github,
        },
        {
          provider: "Oidc" as LinkedLoginProvider,
          enabled: !!options?.oidc,
          linked: !!user?.linked_logins?.Oidc,
        },
      ].filter(
        ({ enabled, provider }) => enabled && user.config.type !== provider
      ),
    [user, options]
  );
  const linkWithOauth = useLinkWithOauth();
  return (
    <div className="flex flex-col gap-6">
      {/* Profile */}
      <Section title="Profile" icon={<User className="w-4 h-4" />}>
        <Card>
          <CardHeader className="gap-4">
            {/* Profile Info */}
            <UserProfile user={user} />

            {/* Update Username */}
            <div className="flex items-center gap-4 flex-wrap">
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
            {options?.local && (
              <div className="flex items-center gap-4 flex-wrap">
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

      {/* Linked Logins */}
      {!!thirdPartyProviders.length && (
        <Section title="Linked Logins" icon={<KeyRound className="w-4 h-4" />}>
          <DataTable
            tableKey="linked-logins-v1"
            data={thirdPartyProviders}
            columns={[
              { header: "Provider", accessorKey: "provider" },
              {
                header: "Linked",
                cell: ({
                  row: {
                    original: { linked },
                  },
                }) => (
                  <StatusBadge
                    text={linked ? "Linked" : "Unlinked"}
                    intent={linked ? "Good" : "Critical"}
                  />
                ),
              },
              {
                header: "Link",
                cell: ({
                  row: {
                    original: { provider, linked },
                  },
                }) =>
                  linked ? (
                    <ConfirmButton
                      title={`Unlink ${provider}`}
                      icon={<Trash className="w-4 h-4" />}
                      variant="destructive"
                      onClick={() => unlink({ provider })}
                    />
                  ) : provider === "Local" ? (
                    <>Set password above to enable.</>
                  ) : (
                    <Button
                      variant="outline"
                      className="flex gap-2 px-3 items-center"
                      onClick={() => linkWithOauth(provider)}
                      disabled={!!user.linked_logins?.[provider]}
                    >
                      Link {provider}
                      <CirclePlus className="w-4 h-4" />
                    </Button>
                  ),
              },
            ]}
          />
        </Section>
      )}

      {/* 2FA */}
      <Section title="2FA" icon={<KeyRound className="w-4 h-4" />}>
        <div className="flex items-center gap-x-6 gap-y-4 flex-wrap">
          <EnrollPasskey user={user} />
          <EnrollTotp user={user} />
          <div
            className={cn(
              "flex items-center gap-2",
              !user.passkey?.created_at && !user.totp?.confirmed_at && "hidden"
            )}
          >
            <Switch
              id="update-skip-2fa"
              checked={user.third_party_skip_2fa}
              onCheckedChange={(skip) => update_skip_2fa({ skip })}
            />
            <Label
              htmlFor="update-skip-2fa"
              className="flex items-center gap-2 cursor-pointer"
            >
              Skip 2FA for third party logins
              <StatusBadge
                text={user.third_party_skip_2fa ? "ENABLED" : "DISABLED"}
                intent={user.third_party_skip_2fa ? "Good" : "Critical"}
              />
            </Label>
          </div>
        </div>
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

      <div className="font-mono text-muted-foreground">2FA:</div>
      {user.passkey?.created_at
        ? "Passkey"
        : user.totp?.confirmed_at
          ? "TOTP"
          : "None"}

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
