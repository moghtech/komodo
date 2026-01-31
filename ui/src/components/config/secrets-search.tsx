import { useRead } from "@/lib/hooks";
import { UsableResource } from "@/resources";
import { Group } from "@mantine/core";
import SecretSelector from "./secret-selector";

export interface SecretsSearchProps {
  /** ID or name */
  server?: string;
  /** ID or name */
  builder?: string;
}

export default function SecretsSearch({ server, builder }: SecretsSearchProps) {
  const variables = useRead("ListVariables", {}).data ?? [];
  const secretQuery = server
    ? { target: { type: "Server" as UsableResource, id: server } }
    : builder
      ? { target: { type: "Builder" as UsableResource, id: builder } }
      : {};
  const secrets = useRead("ListSecrets", secretQuery).data ?? [];

  if (variables.length === 0 && secrets.length === 0) {
    return;
  }

  return (
    <Group>
      {variables.length > 0 && (
        <SecretSelector
          type="Variable"
          keys={variables.map((v) => v.name)}
          position="right-start"
        />
      )}
      {variables.length > 0 && (
        <SecretSelector type="Secret" keys={secrets} position="right-start" />
      )}
    </Group>
  );
}
