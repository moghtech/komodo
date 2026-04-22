import { useRead } from "@/lib/hooks";
import { ConfigItem } from "@/ui/config/item";
import { Button, Group, Select, SelectProps, TextInput } from "@mantine/core";
import { useState } from "react";

export type ProviderSelectorAccountType = "git" | "docker";

export interface ProviderSelectorProps extends Omit<SelectProps, "onSelect"> {
  disabled: boolean;
  accountType: ProviderSelectorAccountType;
  selected: string | undefined;
  onSelect: (provider: string) => void;
  showCustom?: boolean;
  showLabel?: boolean;
}

export default function ProviderSelector({
  accountType,
  selected,
  onSelect,
  disabled,
  showCustom,
  showLabel,
  ...selectorProps
}: ProviderSelectorProps) {
  const [dbRequest, configRequest]:
    | ["ListGitProviderAccounts", "ListGitProvidersFromConfig"]
    | ["ListDockerRegistryAccounts", "ListDockerRegistriesFromConfig"] =
    accountType === "git"
      ? ["ListGitProviderAccounts", "ListGitProvidersFromConfig"]
      : ["ListDockerRegistryAccounts", "ListDockerRegistriesFromConfig"];
  const dbProviders = useRead(dbRequest, {}).data;
  const configProviders = useRead(configRequest, {}).data;
  const [customMode, setCustomMode] = useState(false);

  if (customMode) {
    return (
      <TextInput
        label={showLabel && "Domain"}
        placeholder="Input custom provider domain"
        w={{ base: "85%", lg: 400 }}
        value={selected}
        onChange={(e) => onSelect(e.target.value)}
        onBlur={() => setCustomMode(false)}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            setCustomMode(false);
          }
        }}
        autoFocus
      />
    );
  }

  const domains = new Set<string>();
  selected && domains.add(selected);
  for (const provider of dbProviders ?? []) {
    domains.add(provider.domain);
  }
  for (const provider of configProviders ?? []) {
    domains.add(provider.domain);
  }
  const providers = ["None", ...domains];
  providers.sort();

  if (showCustom) {
    providers.push("Custom");
  }

  return (
    <Select
      placeholder="Select Provider"
      label={showLabel && "Domain"}
      value={selected || "None"}
      disabled={disabled}
      data={providers}
      onChange={(value) => {
        if (value === "Custom") {
          onSelect("");
          setCustomMode(true);
        } else if (value === "None") {
          onSelect("");
        } else if (value) {
          onSelect(value);
        }
      }}
      {...selectorProps}
    />
  );
}

export function ProviderSelectorConfig({
  description,
  https,
  onHttpsSwitch,
  accountType,
  ...props
}: {
  description?: string;
  https?: boolean;
  onHttpsSwitch?: () => void;
} & ProviderSelectorProps) {
  const select = accountType === "git" ? "git provider" : "docker registry";
  const label = accountType === "git" ? "Git Provider" : "Image Registry";
  const selector = (
    <ProviderSelector accountType={accountType} w="fit-content" {...props} />
  );
  return (
    <ConfigItem
      label={label}
      description={description ?? `Select ${select} domain`}
    >
      {accountType === "git" ? (
        <Group>
          <Button
            onClick={onHttpsSwitch}
            disabled={props.disabled}
            px="sm"
            py="xs"
          >
            {`http${https ? "s" : ""}://`}
          </Button>
          {selector}
        </Group>
      ) : (
        selector
      )}
    </ConfigItem>
  );
}
