/* eslint-disable @typescript-eslint/no-explicit-any */
import { ResourceSelector } from "@components/resources/common";
import { fmt_date, fmt_version } from "@lib/formatting";
import { useRead } from "@lib/hooks";
import { filterBySplit } from "@lib/utils";
import { Types } from "komodo_client";
import { CaretSortIcon } from "@radix-ui/react-icons";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@ui/command";
import { Input } from "@ui/input";
import { Popover, PopoverContent, PopoverTrigger } from "@ui/popover";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { SearchX } from "lucide-react";
import { useState } from "react";

const BuildVersionSelector = ({
  disabled,
  buildId,
  selected,
  onSelect,
}: {
  disabled: boolean;
  buildId: string | undefined;
  selected: Types.Version | undefined;
  onSelect: (version: Types.Version) => void;
}) => {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const versions = useRead(
    "ListBuildVersions",
    { build: buildId! },
    { enabled: !!buildId }
  ).data;
  const filtered = filterBySplit(versions, search, (item) =>
    fmt_version(item.version)
  );
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild disabled={disabled}>
        <div className="h-full w-[150px] cursor-pointer flex items-center justify-between whitespace-nowrap rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1">
          {selected ? fmt_version(selected) : "Latest"}
          <CaretSortIcon className="h-4 w-4 opacity-50" />
        </div>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-[200px] max-h-[200px] p-0">
        <Command shouldFilter={false}>
          <CommandInput
            placeholder="Search Versions"
            value={search}
            onValueChange={setSearch}
            className="h-9"
          />
          <CommandList>
            <CommandEmpty className="flex justify-evenly items-center">
              No Versions Found
              <SearchX className="w-3 h-3" />
            </CommandEmpty>

            <CommandGroup>
              <CommandItem
                className="cursor-pointer"
                onSelect={() => {
                  onSelect({ major: 0, minor: 0, patch: 0 });
                  setOpen(false);
                }}
              >
                <div>Latest</div>
              </CommandItem>
              {filtered?.map((v) => {
                const version = fmt_version(v.version);
                return (
                  <CommandItem
                    key={version}
                    onSelect={() => {
                      onSelect(v.version);
                      setOpen(false);
                    }}
                    className="flex items-center justify-between cursor-pointer"
                  >
                    <div>{version}</div>
                    <div className="text-muted-foreground">
                      {fmt_date(new Date(v.ts))}
                    </div>
                  </CommandItem>
                );
              })}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
};

const ImageTypeSelector = ({
  selected,
  onSelect,
  disabled,
}: {
  selected: Types.DeploymentImage["type"] | undefined;
  onSelect: (type: Types.DeploymentImage["type"]) => void;
  disabled: boolean;
}) => (
  <Select
    value={selected || undefined}
    onValueChange={onSelect}
    disabled={disabled}
  >
    <SelectTrigger className="max-w-[150px]" disabled={disabled}>
      <SelectValue placeholder="Select Type" />
    </SelectTrigger>
    <SelectContent>
      <SelectItem value={"Image"}>Image</SelectItem>
      <SelectItem value={"Build"}>Build</SelectItem>
    </SelectContent>
  </Select>
);

export const ImageConfig = ({
  image,
  set,
  disabled,
}: {
  image: Types.DeploymentImage | undefined;
  set: (input: Partial<Types.DeploymentConfig>) => void;
  disabled: boolean;
}) => (
  <div className="flex gap-4 w-full items-center">
    <ImageTypeSelector
      selected={image?.type}
      disabled={disabled}
      onSelect={(type) =>
        set({
          image: {
            type: type,
            params:
              type === "Image"
                ? { image: "" }
                : ({
                    build_id: "",
                    version: { major: 0, minor: 0, patch: 0 },
                  } as any),
          },
        })
      }
    />
    {image?.type === "Build" && (
      <>
        <ResourceSelector
          type="Build"
          selected={image.params.build_id}
          onSelect={(id) =>
            set({
              image: {
                ...image,
                params: { ...image.params, build_id: id },
              },
            })
          }
          disabled={disabled}
        />
        <BuildVersionSelector
          buildId={image.params.build_id}
          selected={image.params.version}
          onSelect={(version) =>
            set({
              image: {
                ...image,
                params: {
                  ...image.params,
                  version,
                },
              },
            })
          }
          disabled={disabled}
        />
      </>
    )}
    {image?.type === "Image" && (
      <Input
        value={image.params.image}
        onChange={(e) =>
          set({
            image: {
              ...image,
              params: { image: e.target.value },
            },
          })
        }
        className="w-full"
        placeholder="image name"
        disabled={disabled}
      />
    )}
  </div>
);
