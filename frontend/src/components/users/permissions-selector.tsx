import { useState } from "react";
import { UsableResource } from "@types";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { Types } from "komodo_client";
import { Button } from "@ui/button";
import { filterBySplit } from "@lib/utils";
import { Popover, PopoverContent, PopoverTrigger } from "@ui/popover";
import { fmt_upper_camelcase } from "@lib/formatting";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@ui/command";
import { SearchX } from "lucide-react";
import { Checkbox } from "@ui/checkbox";

export const PermissionLevelSelector = ({
  level,
  onSelect,
  disabled,
}: {
  level: Types.PermissionLevel;
  onSelect: (level: Types.PermissionLevel) => void;
  disabled?: boolean;
}) => {
  return (
    <Select
      value={level}
      onValueChange={(value) => onSelect(value as Types.PermissionLevel)}
      disabled={disabled}
    >
      <SelectTrigger className="w-32 capitalize" disabled={disabled}>
        <SelectValue />
      </SelectTrigger>
      <SelectContent className="w-32">
        {Object.keys(Types.PermissionLevel).map((permission) => (
          <SelectItem
            value={permission}
            key={permission}
            className="capitalize"
            disabled={disabled}
          >
            {permission}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
};

const ALL_PERMISSIONS_BY_TYPE: {
  [type: string]: Types.SpecificPermission[] | undefined;
} = {
  Swarm: [
    Types.SpecificPermission.Attach,
    Types.SpecificPermission.Inspect,
    Types.SpecificPermission.Logs,
  ],
  Server: [
    Types.SpecificPermission.Attach,
    Types.SpecificPermission.Inspect,
    Types.SpecificPermission.Logs,
    Types.SpecificPermission.Terminal,
    Types.SpecificPermission.Processes,
  ],
  Stack: [
    Types.SpecificPermission.Inspect,
    Types.SpecificPermission.Logs,
    Types.SpecificPermission.Terminal,
  ],
  Deployment: [
    Types.SpecificPermission.Inspect,
    Types.SpecificPermission.Logs,
    Types.SpecificPermission.Terminal,
  ],
  Build: [Types.SpecificPermission.Attach],
  Repo: [Types.SpecificPermission.Attach],
  Builder: [Types.SpecificPermission.Attach],
};

export const SpecificPermissionSelector = ({
  open,
  onOpenChange,
  type,
  specific,
  onSelect,
  disabled,
}: {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  type: UsableResource;
  specific: Types.SpecificPermission[];
  onSelect: (permission: Types.SpecificPermission) => void;
  disabled?: boolean;
}) => {
  const [search, setSearch] = useState("");
  const all_permissions = ALL_PERMISSIONS_BY_TYPE[type];
  // These resources don't have any specific permissions to add
  if (!all_permissions) {
    return (
      <Button
        variant="outline"
        className="px-3 py-2 rounded-md text-sm w-full justify-start cursor-not-allowed"
        disabled
      >
        N/a
      </Button>
    );
  }
  const filtered = filterBySplit(all_permissions, search, (item) => item);
  return (
    <Popover open={open} onOpenChange={onOpenChange}>
      <PopoverTrigger asChild disabled={disabled}>
        <Button
          variant="outline"
          className="px-3 py-2 rounded-md text-sm w-full justify-start"
          disabled={disabled}
        >
          {!specific.length
            ? "None"
            : specific.map(fmt_upper_camelcase).join(", ")}
        </Button>
      </PopoverTrigger>
      <PopoverContent className="p-1" align="start" sideOffset={12}>
        <Command shouldFilter={false} loop>
          <CommandInput
            placeholder={"Search Permissions"}
            className="h-9"
            value={search}
            onValueChange={setSearch}
          />
          <CommandList>
            <CommandEmpty className="flex justify-evenly items-center pt-2">
              No Permissions Found
              <SearchX className="w-3 h-3" />
            </CommandEmpty>
            <CommandGroup>
              {filtered.map((permission) => (
                <CommandItem
                  key={permission}
                  onSelect={() => onSelect(permission)}
                  className="flex items-center justify-between cursor-pointer"
                >
                  <div className="p-1">{fmt_upper_camelcase(permission)}</div>
                  <Checkbox checked={specific.includes(permission)} />
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
};
