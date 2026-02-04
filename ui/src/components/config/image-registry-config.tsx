import { useRead } from "@/lib/hooks";
import { ConfigItem } from "@/ui/config/item";
import { ActionIcon, Badge, Group, Text } from "@mantine/core";
import { Types } from "komodo_client";
import ProviderSelector from "./provider-selector";
import AccountSelector from "./account-selector";
import OrganizationSelector from "./organization-selector";
import { ICONS } from "@/theme/icons";

export interface ImageRegistryConfig {
  registry: Types.ImageRegistryConfig | undefined;
  setRegistry: (registry: Types.ImageRegistryConfig) => void;
  disabled: boolean;
  builderId: string | undefined;
  onRemove: () => void;
  imageName: string | undefined;
}

export default function ImageRegistryConfig({
  registry,
  setRegistry,
  disabled,
  builderId,
  onRemove,
  imageName,
}: ImageRegistryConfig) {
  // This is the only way to get organizations for now
  const config_provider = useRead("ListDockerRegistriesFromConfig", {
    target: builderId ? { type: "Builder", id: builderId } : undefined,
  }).data?.find((provider) => {
    return provider.domain === registry?.domain;
  });

  const organizations = config_provider?.organizations ?? [];
  const namespace = registry?.organization || registry?.account;

  return (
    <ConfigItem
      label={
        <Group>
          <Text c="dimmed">Pushes to:</Text>
          {registry?.domain && registry.domain + " / "}
          {registry?.domain && (namespace ? namespace : "<namespace>") + " / "}
          {imageName}
          {!registry?.domain && <Badge variant="light">Local</Badge>}
        </Group>
      }
      gap="xs"
    >
      <Group>
        <ProviderSelector
          disabled={disabled}
          accountType="docker"
          selected={registry?.domain}
          onSelect={(domain) =>
            setRegistry({
              ...registry,
              domain,
            })
          }
          showCustom={false}
          showLabel
        />
        <AccountSelector
          id={builderId}
          type="Builder"
          accountType="docker"
          provider={registry?.domain!}
          selected={registry?.account}
          onSelect={(account) =>
            setRegistry({
              ...registry,
              account,
            })
          }
          disabled={!registry?.domain || disabled}
          showLabel
        />
        <OrganizationSelector
          organizations={organizations}
          selected={registry?.organization!}
          onSelect={(organization) =>
            setRegistry({
              ...registry,
              organization,
            })
          }
          disabled={disabled}
          showLabel
        />
        {!disabled && (
          <ActionIcon
            color="red"
            onClick={onRemove}
            style={{ alignSelf: "flex-end" }}
            mb={4}
          >
            <ICONS.Remove size="1rem" />
          </ActionIcon>
        )}
      </Group>
    </ConfigItem>
  );
}
