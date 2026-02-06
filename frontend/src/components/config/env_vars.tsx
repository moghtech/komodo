import { SecretSelector } from "@components/config/util";
import { copyToClipboard } from "@components/util";
import { useRead } from "@lib/hooks";
import { Types } from "komodo_client";
import { useToast } from "@ui/use-toast";

export const SecretsSearch = ({
  server,
}: {
  /// eg server id
  server?: string;
}) => {
  if (server) return <SecretsWithServer server={server} />;
  return <SecretsNoServer />;
};

const SecretsNoServer = () => {
  const variables = useRead("ListVariables", {}).data ?? [];
  const secrets = useRead("ListSecrets", {}).data ?? [];
  return <SecretsView variables={variables} secrets={secrets} />;
};

const SecretsWithServer = ({
  server,
}: {
  /// eg server id
  server: string;
}) => {
  const variables = useRead("ListVariables", {}).data ?? [];
  const secrets =
    useRead("ListSecrets", { target: { type: "Server", id: server } }).data ??
    [];
  return <SecretsView variables={variables} secrets={secrets} />;
};

const SecretsView = ({
  variables,
  secrets,
}: {
  variables: Types.ListVariablesResponse;
  secrets: Types.ListSecretsResponse;
}) => {
  const { toast } = useToast();
  if (variables.length === 0 && secrets.length === 0) return;
  return (
    <div className="flex items-center gap-2">
      {variables.length > 0 && (
        <SecretSelector
          type="Variable"
          keys={variables.map((v) => v.name)}
          onSelect={async (variable) => {
            if (!variable) return;
            const success = await copyToClipboard("[[" + variable + "]]");
            if (success) {
              toast({ title: "Copied selection" });
            }
          }}
          disabled={false}
          side="right"
          align="start"
        />
      )}
      {secrets.length > 0 && (
        <SecretSelector
          type="Secret"
          keys={secrets}
          onSelect={async (secret) => {
            if (!secret) return;
            const success = await copyToClipboard("[[" + secret + "]]");
            if (success) {
              toast({ title: "Copied selection" });
            }
          }}
          disabled={false}
          side="right"
          align="start"
        />
      )}
    </div>
  );
};
