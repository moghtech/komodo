import { MonacoEditor } from "mogh_ui";
import { usePermissions, useRead, useWrite } from "@/lib/hooks";
import { Config, ConfigList } from "mogh_ui";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";

export default function AwsBuilderConfig({ id }: { id: string }) {
  const { canWrite } = usePermissions({ type: "Builder", id });
  const config = useRead("GetBuilder", { builder: id }).data?.config
    ?.params as Types.AwsBuilderConfig;
  const global_disabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const [update, setUpdate] = useLocalStorage<Partial<Types.AwsBuilderConfig>>({
    key: `aws-builder-${id}-update-v1`,
    defaultValue: {},
  });
  const { mutateAsync } = useWrite("UpdateBuilder");
  if (!config) return null;

  const disabled = global_disabled || !canWrite;

  return (
    <Config
      disabled={disabled}
      original={config}
      update={update}
      setUpdate={setUpdate}
      onSave={async () => {
        await mutateAsync({ id, config: { type: "Aws", params: update } });
      }}
      groups={{
        "": [
          {
            label: "General",
            fields: {
              region: {
                description:
                  "Configure the AWS region to launch the instance in.",
                placeholder: "Input region",
              },
              instance_type: {
                description: "Choose the instance type to launch",
                placeholder: "Input instance type",
              },
              ami_id: {
                description:
                  "Create an AMI with Docker and Komodo Periphery installed.",
                placeholder: "Input AMI ID",
              },
              volume_gb: {
                description: "The size of the disk to attach to the instance.",
                placeholder: "Input size",
              },
              key_pair_name: {
                description: "Attach a key pair to the instance",
                placeholder: "Input key pair name",
              },
            },
          },
          {
            label: "Network",
            fields: {
              subnet_id: {
                description: "Configure the subnet to launch the instance in.",
                placeholder: "Input subnet ID",
              },
              security_group_ids: (values, set) => (
                <ConfigList
                  label="Security Group IDs"
                  description="Attach security groups to the instance."
                  field="security_group_ids"
                  values={values ?? []}
                  set={set}
                  disabled={disabled}
                  placeholder="Input ID"
                />
              ),
              assign_public_ip: {
                description:
                  "Whether to assign a public IP to the build instance.",
              },
              use_public_ip: {
                description:
                  "Whether to connect to the instance over the public IP. Otherwise, will use the internal IP.",
              },
            },
          },
          {
            label: "User Data",
            description: "Run a script to setup the instance.",
            fields: {
              user_data: (user_data, set) => {
                return (
                  <MonacoEditor
                    value={user_data}
                    language="shell"
                    onValueChange={(user_data) => set({ user_data })}
                    readOnly={disabled}
                  />
                );
              },
            },
          },
        ],
        additional: [
          {
            label: "Connection",
            labelHidden: true,
            fields: {
              periphery_public_key: {
                label: "Periphery Public Key",
                description:
                  "If provided, the associated private key must be set as Periphery 'private_key'.",
                placeholder: "custom-public-key",
              },
              port: {
                description: "Configure the port to connect to Periphery on.",
                placeholder: "Input port",
              },
              use_https: {
                description: "Whether to connect to Periphery using HTTPS.",
              },
              insecure_tls: {
                description:
                  "Skip Periphery TLS certificate validation when HTTPS is enabled.",
              },
            },
          },
        ],
      }}
    />
  );
}
