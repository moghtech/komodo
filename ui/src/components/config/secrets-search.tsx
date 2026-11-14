import { useRead } from "@/lib/hooks";
import { UsableResource } from "@/resources";
import ExternalSecretPicker from "@/components/secret-provider/external-secret-picker";
import { Group } from "@mantine/core";
import { Types } from "komodo_client";
import SecretSelector from "./secret-selector";

export interface SecretsSearchProps {
  /** ID or name */
  server?: string;
  /** ID or name */
  builder?: string;
  /**
   * Optional resource target ONLY for fields supporting external `[[vault.<slug>.<key>]]`
   * interpolation (`Stack.environment`, `Deployment.environment`, `Build.secret_args`).
   */
  externalResourceTarget?: Types.ResourceTarget;
  onInsertExternalLine?: (line: string) => void;
  disabled?: boolean;
}

export default function SecretsSearch({
  server,
  builder,
  externalResourceTarget,
  onInsertExternalLine,
  disabled,
}: SecretsSearchProps) {
  const variables = useRead("ListVariables", {}).data ?? [];
  const secretQuery = server
    ? { target: { type: "Server" as UsableResource, id: server } }
    : builder
      ? { target: { type: "Builder" as UsableResource, id: builder } }
      : {};
  const secrets = useRead("ListSecrets", secretQuery).data ?? [];

  if (
    variables.length === 0 &&
    secrets.length === 0 &&
    !externalResourceTarget
  ) {
    return;
  }

  return (
    <Group>
      {variables.length > 0 && (
        <SecretSelector
          type="Variable"
          keys={variables.map((v) => v.name)}
          position="bottom-start"
          disabled={disabled}
        />
      )}
      {secrets.length > 0 && (
        <SecretSelector
          type="Secret"
          keys={secrets}
          position="bottom-start"
          disabled={disabled}
        />
      )}
      {externalResourceTarget && (
        <ExternalSecretPicker
          target={externalResourceTarget}
          disabled={disabled}
          onInsertLine={onInsertExternalLine}
        />
      )}
    </Group>
  );
}
