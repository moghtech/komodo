/* eslint-disable @typescript-eslint/no-explicit-any */
import {
  WebhookIdOrName,
  useCtrlKeyListener,
  useRead,
  useWebhookIdOrName,
  WebhookIntegration,
  useWebhookIntegrations,
  useSettingsView,
  usePromptHotkeys,
} from "@lib/hooks";
import { Types } from "komodo_client";
import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from "@ui/select";
import { Button } from "@ui/button";
import { Input } from "@ui/input";
import { Switch } from "@ui/switch";
import {
  CheckCircle,
  MinusCircle,
  PlusCircle,
  Save,
  Search,
  SearchX,
  SquareArrowOutUpRight,
} from "lucide-react";
import { ReactNode, useState } from "react";
import { cn, env_to_text, filterBySplit } from "@lib/utils";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@ui/dialog";
import { snake_case_to_upper_space_case } from "@lib/formatting";
import { ConfirmButton, ShowHideButton } from "@components/util";
import { Popover, PopoverContent, PopoverTrigger } from "@ui/popover";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@ui/command";

import { Card, CardContent, CardHeader } from "@ui/card";
import {
  soft_text_color_class_by_intention,
  text_color_class_by_intention,
} from "@lib/color";
import {
  MonacoDiffEditor,
  MonacoEditor,
  MonacoLanguage,
} from "@components/monaco";
import { useNavigate } from "react-router-dom";

export const ConfigItem = ({
  label,
  boldLabel,
  description,
  children,
  className,
}: {
  label?: ReactNode;
  boldLabel?: boolean;
  description?: ReactNode;
  children: ReactNode;
  className?: string;
}) => (
  <div
    className={cn(
      "pb-6 border-b flex flex-col gap-4 first:pt-0 last:border-b-0 last:pb-0",
      className
    )}
  >
    {(label || description) && (
      <div>
        {label && typeof label === "string" && (
          <div className={cn("capitalize", boldLabel && "font-bold")}>
            {label.split("_").join(" ")}
          </div>
        )}
        {label && typeof label !== "string" && label}
        {description && (
          <div className="text-sm text-muted-foreground">{description}</div>
        )}
      </div>
    )}
    {children}
  </div>
);

export const ConfigInput = ({
  label,
  boldLabel,
  value,
  description,
  disabled,
  placeholder,
  onChange,
  onBlur,
  className,
  inputLeft,
  inputRight,
}: {
  label: string;
  boldLabel?: boolean;
  value: string | number | undefined;
  description?: ReactNode;
  disabled?: boolean;
  placeholder?: string;
  onChange?: (value: string) => void;
  onBlur?: (value: string) => void;
  className?: string;
  inputLeft?: ReactNode;
  inputRight?: ReactNode;
}) => (
  <ConfigItem label={label} boldLabel={boldLabel} description={description}>
    {inputLeft || inputRight ? (
      <div className="flex gap-2 items-center">
        {inputLeft}
        <Input
          className={cn("max-w-[75%] lg:max-w-[400px]", className)}
          type={typeof value === "number" ? "number" : undefined}
          value={value}
          onChange={(e) => onChange && onChange(e.target.value)}
          onBlur={(e) => onBlur && onBlur(e.target.value)}
          placeholder={placeholder}
          disabled={disabled}
        />
        {inputRight}
      </div>
    ) : (
      <Input
        className={cn("max-w-[75%] lg:max-w-[400px]", className)}
        type={typeof value === "number" ? "number" : undefined}
        value={value}
        onChange={(e) => onChange && onChange(e.target.value)}
        onBlur={(e) => onBlur && onBlur(e.target.value)}
        placeholder={placeholder}
        disabled={disabled}
      />
    )}
  </ConfigItem>
);

export const ConfigSwitch = ({
  label,
  boldLabel,
  value: checked,
  description,
  disabled,
  onChange,
}: {
  label: string;
  boldLabel?: boolean;
  value: boolean | undefined;
  description?: ReactNode;
  disabled: boolean;
  onChange: (value: boolean) => void;
}) => (
  <ConfigItem
    label={label}
    description={description}
    boldLabel={boldLabel}
    className="flex-col"
  >
    <div
      className="py-2 flex flex-row gap-4 items-center text-sm cursor-pointer"
      onClick={() => !disabled && onChange(!checked)}
    >
      {/* <div
        className={cn(
          "transition-colors text-muted-foreground",
          !checked && soft_text_color_class_by_intention("Critical")
          // checked && "text-muted-foreground"
        )}
      >
        DISABLED
      </div> */}
      <Switch checked={checked} disabled={disabled} />
      <div
        className={cn(
          "transition-colors",
          soft_text_color_class_by_intention(checked ? "Good" : "Critical")
          // !checked && "text-muted-foreground"
        )}
      >
        {checked ? "ENABLED" : "DISABLED"}
      </div>
    </div>
  </ConfigItem>
);

export const ProviderSelector = ({
  disabled,
  account_type,
  selected,
  onSelect,
  showCustom = true,
}: {
  disabled: boolean;
  account_type: "git" | "docker";
  selected: string | undefined;
  onSelect: (provider: string) => void;
  showCustom?: boolean;
}) => {
  const [db_request, config_request]:
    | ["ListGitProviderAccounts", "ListGitProvidersFromConfig"]
    | ["ListDockerRegistryAccounts", "ListDockerRegistriesFromConfig"] =
    account_type === "git"
      ? ["ListGitProviderAccounts", "ListGitProvidersFromConfig"]
      : ["ListDockerRegistryAccounts", "ListDockerRegistriesFromConfig"];
  const db_providers = useRead(db_request, {}).data;
  const config_providers = useRead(config_request, {}).data;
  const [customMode, setCustomMode] = useState(false);

  if (customMode) {
    return (
      <Input
        placeholder="Input custom provider domain"
        value={selected}
        onChange={(e) => onSelect(e.target.value)}
        className="max-w-[75%] lg:max-w-[400px]"
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
  for (const provider of db_providers ?? []) {
    domains.add(provider.domain);
  }
  for (const provider of config_providers ?? []) {
    domains.add(provider.domain);
  }
  const providers = [...domains];
  providers.sort();

  return (
    <Select
      value={selected}
      onValueChange={(value) => {
        if (value === "Custom") {
          onSelect("");
          setCustomMode(true);
        } else if (value === "None") {
          onSelect("");
        } else {
          onSelect(value);
        }
      }}
      disabled={disabled}
    >
      <SelectTrigger
        className="w-full lg:w-[200px] max-w-[50%]"
        disabled={disabled}
      >
        <SelectValue placeholder="Select Provider" />
      </SelectTrigger>
      <SelectContent>
        {providers
          ?.filter((provider) => provider)
          .map((provider) => (
            <SelectItem key={provider} value={provider}>
              {provider}
            </SelectItem>
          ))}
        {providers !== undefined &&
          selected &&
          !providers.includes(selected) && (
            <SelectItem value={selected}>{selected}</SelectItem>
          )}
        {showCustom && <SelectItem value="Custom">Custom</SelectItem>}
        {!showCustom && <SelectItem value="None">None</SelectItem>}
      </SelectContent>
    </Select>
  );
};

export const ProviderSelectorConfig = (params: {
  disabled: boolean;
  account_type: "git" | "docker";
  selected: string | undefined;
  onSelect: (id: string) => void;
  https?: boolean;
  onHttpsSwitch?: () => void;
  description?: string;
  boldLabel?: boolean;
}) => {
  const select =
    params.account_type === "git" ? "git provider" : "docker registry";
  const label =
    params.account_type === "git" ? "Git Provider" : "Image Registry";
  return (
    <ConfigItem
      label={label}
      description={params.description ?? `Select ${select} domain`}
      boldLabel={params.boldLabel}
    >
      {params.account_type === "git" ? (
        <div className="flex items-center gap-2 w-[75%]">
          <Button
            variant="outline"
            onClick={params.onHttpsSwitch}
            className="py-0 px-2"
            disabled={params.disabled}
          >
            {`http${params.https ? "s" : ""}://`}
          </Button>
          <ProviderSelector {...params} />
        </div>
      ) : (
        <ProviderSelector {...params} />
      )}
    </ConfigItem>
  );
};

export const AccountSelector = ({
  disabled,
  id,
  type,
  account_type,
  provider,
  selected,
  onSelect,
  placeholder = "Select Account",
}: {
  disabled: boolean;
  type: "Server" | "Builder" | "None";
  id?: string;
  account_type: "git" | "docker";
  provider: string;
  selected: string | undefined;
  onSelect: (id: string) => void;
  placeholder?: string;
}) => {
  const [db_request, config_request]:
    | ["ListGitProviderAccounts", "ListGitProvidersFromConfig"]
    | ["ListDockerRegistryAccounts", "ListDockerRegistriesFromConfig"] =
    account_type === "git"
      ? ["ListGitProviderAccounts", "ListGitProvidersFromConfig"]
      : ["ListDockerRegistryAccounts", "ListDockerRegistriesFromConfig"];
  const config_params =
    type === "None" ? {} : { target: id ? { type, id } : undefined };
  const db_accounts = useRead(db_request, {}).data?.filter(
    (account) => account.domain === provider
  );
  const config_providers = useRead(config_request, config_params).data?.filter(
    (_provider) => _provider.domain === provider
  );

  const _accounts = new Set<string>();
  for (const account of db_accounts ?? []) {
    if (account.username) {
      _accounts.add(account.username);
    }
  }
  for (const provider of config_providers ?? []) {
    for (const account of provider.accounts ?? []) {
      _accounts.add(account.username);
    }
  }
  const accounts = [..._accounts];
  accounts.sort();
  return (
    <Select
      value={selected}
      onValueChange={(value) => {
        onSelect(value === "Empty" ? "" : value);
      }}
      disabled={disabled}
    >
      <SelectTrigger
        className="w-full lg:w-[200px] max-w-[50%]"
        disabled={disabled}
      >
        <SelectValue placeholder={placeholder} />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value={"Empty"}>None</SelectItem>
        {accounts
          ?.filter((account) => account)
          .map((account) => (
            <SelectItem key={account} value={account}>
              {account}
            </SelectItem>
          ))}
      </SelectContent>
    </Select>
  );
};

export const AccountSelectorConfig = (params: {
  disabled: boolean;
  id?: string;
  type: "Server" | "Builder" | "None";
  account_type: "git" | "docker";
  provider: string;
  selected: string | undefined;
  onSelect: (id: string) => void;
  placeholder?: string;
  description?: string;
}) => {
  return (
    <ConfigItem
      label="Account"
      description={
        params.description ??
        "Select the account used to log in to the provider"
      }
    >
      <AccountSelector {...params} />
    </ConfigItem>
  );
};

export const ConfigList = <T extends { [key: string]: unknown }>(
  props: InputListProps<T> & {
    label?: string;
    addLabel?: string;
    boldLabel?: boolean;
    description?: ReactNode;
    configClassname?: string;
  }
) => {
  return (
    <ConfigItem {...{ ...props, className: props.configClassname }}>
      {!props.disabled && (
        <Button
          variant="secondary"
          onClick={() =>
            props.set({
              [props.field]: [...props.values, ""],
            } as Partial<T>)
          }
          className="flex items-center gap-2 w-[200px]"
        >
          <PlusCircle className="w-4 h-4" />
          {(props.addLabel ?? "Add " + props.label?.endsWith("s"))
            ? props.label?.slice(0, -1)
            : props.label}
        </Button>
      )}
      {props.values.length > 0 && <InputList {...props} />}
    </ConfigItem>
  );
};

export type InputListProps<T extends { [key: string]: unknown }> = {
  field: keyof T;
  values: string[];
  disabled: boolean;
  set: (update: Partial<T>) => void;
  placeholder?: string;
  className?: string;
};

export const InputList = <T extends { [key: string]: unknown }>({
  field,
  values,
  disabled,
  set,
  placeholder,
  className,
}: InputListProps<T>) => (
  <div className="flex w-full">
    <div className="flex flex-col gap-4 w-fit">
      {values.map((arg, i) => (
        <div className="w-full flex gap-4" key={i}>
          <Input
            placeholder={placeholder}
            value={arg}
            onChange={(e) => {
              values[i] = e.target.value;
              set({ [field]: [...values] } as Partial<T>);
            }}
            disabled={disabled}
            className={cn("w-[400px] max-w-full", className)}
          />
          {!disabled && (
            <Button
              variant="secondary"
              onClick={() =>
                set({
                  [field]: [...values.filter((_, idx) => idx !== i)],
                } as Partial<T>)
              }
            >
              <MinusCircle className="w-4 h-4" />
            </Button>
          )}
        </div>
      ))}
    </div>
  </div>
);

interface ConfirmUpdateProps<T> {
  previous: T;
  content: Partial<T>;
  onConfirm: () => Promise<void>;
  loading?: boolean;
  disabled: boolean;
  language?: MonacoLanguage;
  file_contents_language?: MonacoLanguage;
  key_listener?: boolean;
}

export function ConfirmUpdate<T>({
  previous,
  content,
  onConfirm,
  loading,
  disabled,
  language,
  file_contents_language,
  key_listener = false,
}: ConfirmUpdateProps<T>) {
  const [open, set] = useState(false);
  
  const handleConfirm = async () => {
    await onConfirm();
    set(false);
  };

  const handleCancel = () => {
    set(false);
  };

  // Keep the existing Ctrl+Enter behavior for backward compatibility
  useCtrlKeyListener("Enter", () => {
    if (!key_listener) return;
    if (open) {
      handleConfirm();
    } else {
      set(true);
    }
  });

  // Add new prompt hotkeys for better UX
  usePromptHotkeys({
    onConfirm: handleConfirm,
    onCancel: handleCancel,
    enabled: open,
    confirmDisabled: disabled || loading,
  });

  return (
    <Dialog open={open} onOpenChange={set}>
      <DialogTrigger asChild>
        <Button
          onClick={() => set(true)}
          disabled={disabled}
          className="flex items-center gap-2 w-100"
        >
          <Save className="w-4 h-4" />
          Save
        </Button>
      </DialogTrigger>
      <DialogContent className="max-w-[800px]">
        <DialogHeader>
          <DialogTitle>Confirm Update</DialogTitle>
        </DialogHeader>
        <div className="flex flex-col gap-4 py-4 my-4 max-h-[70vh] overflow-auto">
          {Object.entries(content).map(([key, val], i) => (
            <ConfirmUpdateItem
              key={i}
              _key={key as any}
              val={val as any}
              previous={previous}
              language={language}
              file_contents_language={file_contents_language}
            />
          ))}
        </div>
        <DialogFooter>
          <ConfirmButton
            title="Update"
            icon={<CheckCircle className="w-4 h-4" />}
            onClick={handleConfirm}
            loading={loading}
          />
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
function ConfirmUpdateItem<T>({
  _key,
  val: _val,
  previous,
  language,
  file_contents_language,
}: {
  _key: keyof T;
  val: T[keyof T];
  previous: T;
  language?: MonacoLanguage;
  file_contents_language?: MonacoLanguage;
}) {
  const [show, setShow] = useState(true);
  const val =
    typeof _val === "string"
      ? _val
      : Array.isArray(_val)
        ? _val.length > 0 &&
          ["string", "number", "boolean"].includes(typeof _val[0])
          ? JSON.stringify(_val)
          : JSON.stringify(_val, null, 2)
        : JSON.stringify(_val, null, 2);
  const prev_val =
    typeof previous[_key] === "string"
      ? previous[_key]
      : _key === "environment" ||
          _key === "build_args" ||
          _key === "secret_args"
        ? (env_to_text(previous[_key] as any) ?? "") // For backward compat with 1.14
        : Array.isArray(previous[_key])
          ? previous[_key].length > 0 &&
            ["string", "number", "boolean"].includes(typeof previous[_key][0])
            ? JSON.stringify(previous[_key])
            : JSON.stringify(previous[_key], null, 2)
          : JSON.stringify(previous[_key], null, 2);
  const showDiff =
    val?.includes("\n") ||
    prev_val?.includes("\n") ||
    Math.max(val?.length ?? 0, prev_val?.length ?? 0) > 30;
  return (
    <div
      className={cn("mr-6 flex flex-col gap-2", val === prev_val && "hidden")}
    >
      <Card>
        <CardHeader className="p-4 flex flex-row justify-between items-center">
          <h1 className={text_color_class_by_intention("Neutral")}>
            {snake_case_to_upper_space_case(_key as string)}
          </h1>
          <ShowHideButton show={show} setShow={setShow} />
        </CardHeader>
        {show && (
          <CardContent>
            {showDiff ? (
              <MonacoDiffEditor
                original={prev_val}
                modified={val}
                language={
                  language ??
                  (["environment", "build_args", "secret_args"].includes(
                    _key as string
                  )
                    ? "key_value"
                    : _key === "file_contents"
                      ? file_contents_language
                      : "json")
                }
              />
            ) : (
              <pre style={{ minHeight: 0 }}>
                <span className={text_color_class_by_intention("Critical")}>
                  {prev_val || "None"}
                </span>{" "}
                <span className="text-muted-foreground">{"->"}</span>{" "}
                <span className={text_color_class_by_intention("Good")}>
                  {val || "None"}
                </span>
              </pre>
            )}
          </CardContent>
        )}
      </Card>
    </div>
  );
}

export const SystemCommand = ({
  value,
  disabled,
  set,
}: {
  value?: Types.SystemCommand;
  disabled: boolean;
  set: (value: Types.SystemCommand) => void;
}) => {
  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center gap-2">
        <div className="text-muted-foreground">Path:</div>
        <Input
          placeholder="Command working directory"
          value={value?.path}
          className="w-[200px] lg:w-[300px]"
          onChange={(e) => set({ ...(value || {}), path: e.target.value })}
          disabled={disabled}
        />
      </div>
      <MonacoEditor
        value={
          value?.command ||
          "  # Add multiple commands on new lines. Supports comments.\n  "
        }
        language="shell"
        onValueChange={(command) => set({ ...(value || {}), command })}
        readOnly={disabled}
      />
    </div>
  );
};

export const AddExtraArgMenu = ({
  onSelect,
  type,
  disabled,
}: {
  onSelect: (suggestion: string) => void;
  type: "Deployment" | "Build" | "Stack" | "StackBuild";
  disabled?: boolean;
}) => {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const suggestions = useRead(`ListCommon${type}ExtraArgs`, {}).data ?? [];

  const filtered = filterBySplit(suggestions, search, (item) => item);

  if (suggestions.length === 0) {
    return (
      <Button
        variant="secondary"
        className="flex items-center gap-2 w-[200px]"
        onClick={() => onSelect("")}
        disabled={disabled}
      >
        <PlusCircle className="w-4 h-4" /> Add Extra Arg
      </Button>
    );
  }

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="secondary"
          className="flex items-center gap-2 w-[200px]"
          disabled={disabled}
        >
          <PlusCircle className="w-4 h-4" /> Add Extra Arg
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[300px] max-h-[400px] p-0" align="start">
        <Command shouldFilter={false}>
          <CommandInput
            placeholder="Search suggestions"
            className="h-9"
            value={search}
            onValueChange={setSearch}
          />
          <CommandList>
            <CommandEmpty className="flex justify-evenly items-center">
              No Suggestions Found
              <SearchX className="w-3 h-3" />
            </CommandEmpty>

            <CommandGroup>
              <CommandItem
                onSelect={() => {
                  onSelect("");
                  setOpen(false);
                }}
                className="w-full cursor-pointer"
              >
                Empty Extra Arg
              </CommandItem>

              {filtered?.map((suggestion) => (
                <CommandItem
                  key={suggestion}
                  onSelect={() => {
                    onSelect(suggestion);
                    setOpen(false);
                  }}
                  className="w-full overflow-hidden overflow-ellipsis cursor-pointer"
                >
                  {suggestion}
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
};

const IMAGE_REGISTRY_DESCRIPTION = "Configure where the built image is pushed.";

export const ImageRegistryConfig = ({
  registry,
  setRegistry,
  disabled,
  resource_id,
}: {
  registry: Types.ImageRegistryConfig | undefined;
  setRegistry: (registry: Types.ImageRegistryConfig) => void;
  disabled: boolean;
  // For builds, its builder id. For servers, its server id.
  resource_id?: string;
}) => {
  // This is the only way to get organizations for now
  const config_provider = useRead("ListDockerRegistriesFromConfig", {
    target: resource_id ? { type: "Builder", id: resource_id } : undefined,
  }).data?.find((provider) => {
    return provider.domain === registry?.domain;
  });

  const organizations = config_provider?.organizations ?? [];

  return (
    <>
      <ConfigItem
        label="Image Registry"
        boldLabel
        description={IMAGE_REGISTRY_DESCRIPTION}
      >
        <div className="flex items-center justify-stretch gap-4">
          <ProviderSelector
            disabled={disabled}
            account_type="docker"
            selected={registry?.domain}
            onSelect={(domain) =>
              setRegistry({
                ...registry,
                domain,
              })
            }
            showCustom={false}
          />
        </div>
      </ConfigItem>

      {registry?.domain && (
        <>
          <ConfigItem
            label="Account"
            description="Select the account used to authenticate against the registry."
          >
            <AccountSelector
              id={resource_id}
              type="Builder"
              account_type="docker"
              provider={registry.domain!}
              selected={registry.account}
              onSelect={(account) =>
                setRegistry({
                  ...registry,
                  account,
                })
              }
              disabled={disabled}
            />
          </ConfigItem>
          <ConfigItem
            label="Organization"
            description="Push the build under an organization / project namespace, rather than the account namespace."
          >
            <OrganizationSelector
              organizations={organizations}
              selected={registry?.organization!}
              set={(organization) =>
                setRegistry({
                  ...registry,
                  organization,
                })
              }
              disabled={disabled}
            />
          </ConfigItem>
        </>
      )}
    </>
  );
};

const OrganizationSelector = ({
  organizations,
  selected,
  set,
  disabled,
}: {
  organizations: string[];
  selected: string;
  set: (org: string) => void;
  disabled: boolean;
}) => {
  const [customMode, setCustomMode] = useState(false);
  if (customMode) {
    return (
      <Input
        placeholder="Input custom organization name"
        value={selected}
        onChange={(e) => set(e.target.value)}
        className="max-w-[75%] lg:max-w-[400px]"
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

  const orgs =
    selected === "" || organizations.includes(selected)
      ? organizations
      : [...organizations, selected];
  orgs.sort();

  return (
    <Select
      value={selected}
      onValueChange={(organization) => {
        if (organization === "Custom") {
          set("");
          setCustomMode(true);
        } else if (organization === "Empty") {
          set("");
        } else {
          set(organization);
        }
      }}
      disabled={disabled}
    >
      <SelectTrigger
        className="w-full lg:w-[200px] max-w-[50%]"
        disabled={disabled}
      >
        <SelectValue placeholder="Select Organization" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value={"Empty"}>None</SelectItem>
        {orgs
          ?.filter((org) => org)
          .map((org) => (
            <SelectItem key={org} value={org}>
              {org}
            </SelectItem>
          ))}
        <SelectItem value={"Custom"}>Custom</SelectItem>
      </SelectContent>
    </Select>
  );
};

export const SecretSelector = ({
  keys,
  onSelect,
  type,
  disabled,
  align = "start",
  side = "right",
}: {
  keys: string[];
  onSelect: (key: string) => void;
  type: "Variable" | "Secret";
  disabled: boolean;
  align?: "start" | "center" | "end";
  side?: "bottom" | "right";
}) => {
  const nav = useNavigate();
  const [_, setSettingsView] = useSettingsView();
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const filtered = filterBySplit(keys, search, (item) => item).sort((a, b) => {
    if (a > b) {
      return 1;
    } else if (a < b) {
      return -1;
    } else {
      return 0;
    }
  });
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button variant="secondary" className="flex gap-2" disabled={disabled}>
          <Search className="w-4 h-4" />
          <div>{type}s</div>
        </Button>
      </PopoverTrigger>
      <PopoverContent
        className="w-[300px] max-h-[300px] p-0"
        align={align}
        side={side}
      >
        <Command shouldFilter={false}>
          <CommandInput
            placeholder={`Search ${type}s`}
            className="h-9"
            value={search}
            onValueChange={setSearch}
          />
          <CommandList>
            <CommandEmpty className="flex justify-evenly items-center pt-2">
              {`No ${type}s Found`}
              <SearchX className="w-3 h-3" />
            </CommandEmpty>

            <CommandGroup>
              {filtered.map((key) => (
                <CommandItem
                  key={key}
                  onSelect={() => {
                    onSelect(key);
                    setOpen(false);
                  }}
                  className="flex items-center justify-between cursor-pointer"
                >
                  <div className="p-1">{key}</div>
                </CommandItem>
              ))}
              <CommandItem
                onSelect={() => {
                  setOpen(false);
                  setSettingsView("Variables");
                  nav("/settings");
                }}
                className="flex items-center justify-between cursor-pointer"
              >
                <div className="p-1">All</div>
                <SquareArrowOutUpRight className="w-4 h-4" />
              </CommandItem>
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
};

export const WebhookBuilder = ({
  git_provider,
  children,
}: {
  git_provider: string;
  children?: ReactNode;
}) => {
  return (
    <ConfigItem>
      <div className="grid items-center grid-cols-[auto_1fr] gap-x-6 gap-y-2 w-fit">
        <div className="text-muted-foreground text-sm">Auth style?</div>
        <WebhookIntegrationSelector git_provider={git_provider} />

        <div className="text-muted-foreground text-sm">
          Resource Id or Name?
        </div>
        <WebhookIdOrNameSelector />

        {children}
      </div>
    </ConfigItem>
  );
};

/** Should call `useWebhookIntegrations` in util/hooks to get the current value */
export const WebhookIntegrationSelector = ({
  git_provider,
}: {
  git_provider: string;
}) => {
  const { integrations, setIntegration } = useWebhookIntegrations();
  const integration = integrations[git_provider]
    ? integrations[git_provider]
    : git_provider === "gitlab.com"
      ? "Gitlab"
      : "Github";
  return (
    <Select
      value={integration}
      onValueChange={(integration) =>
        setIntegration(git_provider, integration as WebhookIntegration)
      }
    >
      <SelectTrigger className="w-[200px]">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {["Github", "Gitlab"].map((integration) => (
          <SelectItem key={integration} value={integration}>
            {integration}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
};

/** Should call `useWebhookIdOrName` in util/hooks to get the current value */
export const WebhookIdOrNameSelector = () => {
  const [idOrName, setIdOrName] = useWebhookIdOrName();
  return (
    <Select
      value={idOrName}
      onValueChange={(idOrName) => setIdOrName(idOrName as WebhookIdOrName)}
    >
      <SelectTrigger className="w-[200px]">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {["Id", "Name"].map((idOrName) => (
          <SelectItem key={idOrName} value={idOrName}>
            {idOrName}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
};
