import { usePermissions, useRead, useWrite } from "@/lib/hooks";
import { Config } from "mogh_ui";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";

export default function UrlBuilderConfig({ id }: { id: string }) {
  const { canWrite } = usePermissions({ type: "Builder", id });
  const config = useRead("GetBuilder", { builder: id }).data?.config;

  const [update, setUpdate] = useLocalStorage<Partial<Types.UrlBuilderConfig>>({
    key: `url-builder-${id}-update-v1`,
    defaultValue: {},
  });
  const { mutateAsync } = useWrite("UpdateBuilder");

  if (!config) return null;

  const disabled = !canWrite;
  const params = config.params as Types.UrlBuilderConfig;
  const address = update.address ?? params.address;
  const tls_address = !!address && !address.startsWith("ws://");

  return (
    <Config
      disabled={disabled}
      original={params}
      update={update}
      setUpdate={setUpdate}
      onSave={async () => {
        await mutateAsync({ id, config: { type: "Url", params: update } });
      }}
      groups={{
        "": [
          {
            label: "General",
            labelHidden: true,
            fields: {
              address: {
                description: "The address of the Periphery agent",
                placeholder: "wss://periphery:8120",
              },
            },
          },
          {
            label: "Connection",
            labelHidden: true,
            fields: {
              periphery_public_key: {
                label: "Periphery Public Key",
                description:
                  "If provided, the associated private key must be set as Periphery 'private_key'. For Periphery -> Core connection, either this or using 'periphery_public_key' in Core config is required for Periphery to be able to connect.",
                placeholder: "custom-public-key",
              },
              insecure_tls: {
                hidden: !tls_address,
                description: "Skip Periphery TLS certificate validation.",
              },
            },
          },
        ],
      }}
    />
  );
}
