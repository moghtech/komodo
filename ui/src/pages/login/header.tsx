import { komodo_client } from "@/lib/hooks";
import {
  Button,
  Group,
  Stack,
  Text,
  useComputedColorScheme,
} from "@mantine/core";
import { MoghAuth } from "komodo_client";
import { KeyRound } from "lucide-react";
import { useLoginOptions } from "mogh_ui";

export default function LoginHeader({
  secondFactorPending,
}: {
  secondFactorPending: boolean;
}) {
  const options = useLoginOptions().data;
  const theme = useComputedColorScheme();
  return (
    <Group justify="space-between">
      <Group gap="sm">
        <img src="/mogh-512x512.png" width={42} height={42} alt="moghtech" />
        <Stack gap="0">
          <Text fz="h2" fw="450" lts="0.1rem">
            KOMODO
          </Text>
          <Text size="md" opacity={0.6} mt={-8}>
            Log In
          </Text>
        </Stack>
      </Group>
      <Group gap="sm">
        {(
          [
            [options?.oidc, "Oidc"],
            [options?.github, "Github"],
            [options?.google, "Google"],
          ] as Array<
            [boolean | undefined, MoghAuth.Types.ExternalLoginProvider]
          >
        ).map(
          ([enabled, provider]) =>
            enabled && (
              <Button
                key={provider}
                onClick={() => komodo_client().auth.externalLogin(provider)}
                leftSection={
                  provider === "Oidc" ? (
                    <KeyRound size="1rem" />
                  ) : (
                    <img
                      src={`/icons/${provider.toLowerCase()}.svg`}
                      alt={provider}
                      style={{
                        width: "1rem",
                        height: "1rem",
                        filter: theme === "dark" ? "invert(1)" : undefined,
                      }}
                    />
                  )
                }
                w={110}
                disabled={secondFactorPending}
              >
                {provider}
              </Button>
            ),
        )}
      </Group>
    </Group>
  );
}
