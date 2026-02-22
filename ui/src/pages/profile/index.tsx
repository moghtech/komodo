import { useLoginOptions, useManageAuth, useUser } from "@/lib/hooks";
import {
  ActionIcon,
  Fieldset,
  Group,
  PasswordInput,
  Text,
  TextInput,
} from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { Save } from "lucide-react";
import { useEffect, useState } from "react";
import { LinkedLogins } from "./linked-logins";
import { EnrollPasskey } from "./passkey";
import { EnrollTotp } from "./totp";
import EnableSwitch from "@/ui/enable-switch";
import PageGuard from "@/ui/page-guard";
import EntityPage from "@/ui/entity-page";
import ApiKeysSection from "@/components/api-keys/section";

export default function Profile() {
  const options = useLoginOptions().data;
  const { data: user, refetch: refetchUser, isPending } = useUser();
  const [username, setUsername] = useState(user?.username);
  useEffect(() => {
    setUsername(user?.username);
  }, [user?.username]);
  const [password, setPassword] = useState("");
  const { mutate: updateUsername } = useManageAuth("UpdateUsername", {
    onSuccess: () => {
      notifications.show({ message: "Username updated." });
      refetchUser();
    },
  });
  const { mutate: updatePassword } = useManageAuth("UpdatePassword", {
    onSuccess: () => {
      notifications.show({ message: "Password updated." });
      setPassword("");
      refetchUser();
    },
  });
  const { mutate: updateExternalSkip2fa } = useManageAuth(
    "UpdateExternalSkip2fa",
    {
      onSuccess: () => {
        notifications.show({
          message: "External login skip 2fa mode updated.",
        });
        refetchUser();
      },
    },
  );

  return (
    <PageGuard isPending={isPending}>
      {user && (
        <EntityPage>
          <Fieldset legend={<Text size="lg">Login</Text>}>
            <Group>
              <Text ff="monospace">Username:</Text>

              <TextInput
                placeholder="Update username"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                w={250}
              />

              <ActionIcon
                onClick={() => username && updateUsername({ username })}
                disabled={!username || username === user.username}
              >
                <Save size="1rem" />
              </ActionIcon>
            </Group>

            {options?.local && (
              <Group mt="sm">
                <Text ff="monospace">Password:</Text>

                <PasswordInput
                  placeholder="Update password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  w={250}
                />

                <ActionIcon
                  onClick={() => updatePassword({ password })}
                  disabled={!password}
                >
                  <Save size="1rem" />
                </ActionIcon>
              </Group>
            )}
          </Fieldset>

          <LinkedLogins user={user} refetchUser={refetchUser} />

          <Fieldset legend={<Text size="lg">2FA</Text>}>
            <Group>
              <EnrollPasskey user={user} />
              <EnrollTotp user={user} />
              {(!!user.totp?.confirmed_at || !!user.passkey?.created_at) && (
                <EnableSwitch
                  label="Skip 2FA for external logins"
                  checked={user.external_skip_2fa}
                  onCheckedChange={(external_skip_2fa) =>
                    updateExternalSkip2fa({ external_skip_2fa })
                  }
                  redDisabled={false}
                />
              )}
            </Group>
          </Fieldset>

          <ApiKeysSection mt="lg" />
        </EntityPage>
      )}
    </PageGuard>
  );
}
