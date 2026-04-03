import { useLoginOptions, useManageAuth, useUser } from "@/lib/hooks";
import {
  ActionIcon,
  Group,
  PasswordInput,
  Stack,
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
import UserHeader from "@/components/user/header";
import Section from "@/ui/section";
import { ICONS } from "@/lib/icons";

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
      notifications.show({ message: "Username updated.", color: "green" });
      refetchUser();
    },
  });
  const { mutate: updatePassword } = useManageAuth("UpdatePassword", {
    onSuccess: () => {
      notifications.show({ message: "Password updated.", color: "green" });
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
          color: "green",
        });
        refetchUser();
      },
    },
  );

  return (
    <PageGuard
      isPending={isPending}
      error={!user ? "User not found" : undefined}
    >
      {user && (
        <EntityPage>
          <UserHeader user={user} />
          <Section
            title="Login"
            titleFz="h3"
            icon={<ICONS.Key size="1.2rem" />}
            withBorder
          >
            <Stack gap="0">
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
            </Stack>
          </Section>

          <LinkedLogins user={user} refetchUser={refetchUser} />

          <Section
            title="2FA"
            titleFz="h3"
            icon={<ICONS.Key size="1.2rem" />}
            withBorder
          >
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
          </Section>

          <ApiKeysSection />
        </EntityPage>
      )}
    </PageGuard>
  );
}
