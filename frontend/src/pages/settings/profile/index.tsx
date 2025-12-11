import { ConfirmButton } from "@components/util";
import { useRead, useSetTitle, useUser, useWrite } from "@lib/hooks";
import { Button } from "@ui/button";
import { useToast } from "@ui/use-toast";
import { Loader2, User, Eye, EyeOff, KeyRound, UserPen } from "lucide-react";
import { useState } from "react";
import { Input } from "@ui/input";
import { ApiKeysTable } from "@components/api-keys/table";
import { Section } from "@components/layouts";
import { Card, CardHeader } from "@ui/card";
import { Types } from "komodo_client";
import { CreateKey, DeleteKey } from "./api-key";
import { EnrollTotp } from "./totp";
import { EnrollPasskey } from "./passkey";

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

      {/* 2FA */}
      <Section title="2FA" icon={<KeyRound className="w-4 h-4" />}>
        <div className="flex items-center gap-4">
          <EnrollPasskey user={user} />
          <EnrollTotp user={user} />
          {/* <StatusBadge
            text={user.totp?.confirmed_at ? "Enrolled" : "Not Enrolled"}
            intent={user.totp?.confirmed_at ? "Good" : "Critical"}
          /> */}
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
