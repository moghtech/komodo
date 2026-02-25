import { useRead } from "@/lib/hooks";
import { Badge, Center, Group, Loader, Stack, Text } from "@mantine/core";

export default function ProvidersFromConfig({
  type,
}: {
  type: "GitProvider" | "DockerRegistry";
}) {
  const accounts = useRead(
    type === "GitProvider"
      ? "ListGitProvidersFromConfig"
      : "ListDockerRegistriesFromConfig",
    {},
  )
    .data?.map((provider) =>
      provider.accounts.map((account) => [provider.domain, account.username]),
    )
    .flat(1);

  if (!accounts) {
    return (
      <Center>
        <Loader size="xl" />
      </Center>
    );
  }

  if (accounts.length === 0) {
    return;
  }

  return (
    <Stack gap="xs">
      <Text c="dimmed">From config file:</Text>
      <Group>
        {accounts.map(([domain, username]) => (
          <Badge variant="default" size="lg" tt="none">
            {domain} - {username}
          </Badge>
        ))}
      </Group>
    </Stack>
  );
}
