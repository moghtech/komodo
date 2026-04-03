import { useRead } from "@/lib/hooks";
import { ConfigItem } from "mogh_ui";
import { Select, SelectProps } from "@mantine/core";

export interface AccountSelectorProps extends Omit<SelectProps, "onSelect"> {
  disabled: boolean;
  type: "Server" | "Builder" | "None";
  id?: string;
  accountType: "git" | "docker";
  provider: string;
  selected: string | undefined;
  onSelect: (id: string) => void;
  placeholder?: string;
  showLabel?: boolean;
}

export default function AccountSelector({
  disabled,
  type,
  id,
  accountType,
  provider,
  selected,
  onSelect,
  placeholder,
  showLabel,
  ...selectProps
}: AccountSelectorProps) {
  const [dbRequest, configRequest]:
    | ["ListGitProviderAccounts", "ListGitProvidersFromConfig"]
    | ["ListDockerRegistryAccounts", "ListDockerRegistriesFromConfig"] =
    accountType === "git"
      ? ["ListGitProviderAccounts", "ListGitProvidersFromConfig"]
      : ["ListDockerRegistryAccounts", "ListDockerRegistriesFromConfig"];
  const dbAccounts = useRead(dbRequest, {}).data?.filter(
    (account) => account.domain === provider,
  );
  const configParams =
    type === "None" ? {} : { target: id ? { type, id } : undefined };
  const configProviders = useRead(configRequest, configParams).data?.filter(
    (_provider) => _provider.domain === provider,
  );

  const _accounts = new Set<string>();
  for (const account of dbAccounts ?? []) {
    if (account.username) {
      _accounts.add(account.username);
    }
  }
  for (const provider of configProviders ?? []) {
    for (const account of provider.accounts ?? []) {
      _accounts.add(account.username);
    }
  }
  const accounts = [..._accounts];
  accounts.sort();
  accounts.unshift("None");

  return (
    <Select
      label={showLabel && "Account"}
      value={selected || "None"}
      onChange={(value) => value && onSelect(value === "None" ? "" : value)}
      disabled={disabled}
      data={accounts}
      w="fit-content"
      placeholder="Select account"
      {...selectProps}
    />
  );
}

export function AccountSelectorConfig({
  description,
  ...props
}: { description?: string } & AccountSelectorProps) {
  return (
    <ConfigItem
      label="Account"
      description={
        description ?? "Select the account used to log in to the provider"
      }
    >
      <AccountSelector {...props} />
    </ConfigItem>
  );
}
