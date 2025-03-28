import {
  useTags,
  useInvalidate,
  useRead,
  useShiftKeyListener,
  useWrite,
} from "@lib/hooks";
import { cn, filterBySplit } from "@lib/utils";
import { Types } from "komodo_client";
import { Badge } from "@ui/badge";
import { Button } from "@ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "@ui/popover";
import { useToast } from "@ui/use-toast";
import { MinusCircle, PlusCircle, SearchX, Tag, X } from "lucide-react";
import { ReactNode, useEffect, useState } from "react";
import { tag_background_class } from "@lib/color";

type TargetExcludingSystem = Exclude<Types.ResourceTarget, { type: "System" }>;

export const TagsFilter = () => {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const { tags, add_tag, remove_tag, clear_tags } = useTags();
  const all_tags = useRead("ListTags", {}).data;
  const filtered = filterBySplit(all_tags, search, (item) => item.name);
  useShiftKeyListener("T", () => setOpen(true));
  useShiftKeyListener("C", () => clear_tags());
  return (
    <div className="flex gap-3 items-center">
      {tags.length > 0 && (
        <Button
          variant="destructive"
          className="px-2 py-1.5 h-fit"
          onClick={() => clear_tags()}
        >
          <X className="w-4 h-4" />
        </Button>
      )}

      <TagsFilterTags tag_ids={tags} onBadgeClick={remove_tag} />

      <Popover
        open={open}
        onOpenChange={(open) => {
          setSearch("");
          setOpen(open);
        }}
      >
        <PopoverTrigger asChild>
          <Button variant="outline" className="flex items-center gap-2">
            <Tag className="w-3 h-3" />
            Tag Filter
          </Button>
        </PopoverTrigger>
        <PopoverContent
          className="w-[200px] max-h-[200px] p-0"
          sideOffset={12}
          align="end"
        >
          <Command shouldFilter={false}>
            <CommandInput
              placeholder="Search Tags"
              className="h-9"
              value={search}
              onValueChange={setSearch}
            />
            <CommandList>
              <CommandEmpty className="flex justify-evenly items-center pt-2">
                No Tags Found
                <SearchX className="w-3 h-3" />
              </CommandEmpty>

              <CommandGroup>
                {filtered
                  ?.filter((tag) => !tags.includes(tag._id!.$oid))
                  .map((tag) => (
                    <CommandItem
                      key={tag.name}
                      onSelect={() => {
                        add_tag(tag._id!.$oid);
                        setSearch("");
                        setOpen(false);
                      }}
                      className="flex items-center justify-between cursor-pointer"
                    >
                      <div className="p-1">{tag.name}</div>
                      <div
                        className={cn(
                          "w-[25px] h-[25px] rounded-sm bg-opacity-70",
                          tag_background_class(tag.color)
                        )}
                      />
                    </CommandItem>
                  ))}
              </CommandGroup>
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>
    </div>
  );
};

export const TagsFilterTags = ({
  tag_ids,
  onBadgeClick,
}: {
  tag_ids?: string[];
  onBadgeClick?: (tag_id: string) => void;
}) => {
  const all_tags = useRead("ListTags", {}).data;
  const get_tag = (tag_id: string) =>
    all_tags?.find((t) => t._id?.$oid === tag_id);
  return (
    <>
      {tag_ids?.map((tag_id) => {
        const tag = get_tag(tag_id);
        const color = tag_background_class(tag?.color);
        return (
          <Badge
            key={tag_id}
            variant="secondary"
            className={cn(
              "flex gap-1 px-2 py-1.5 cursor-pointer text-nowrap bg-opacity-30 hover:bg-opacity-70",
              color,
              `hover:${color}`
            )}
            onClick={() => onBadgeClick && onBadgeClick(tag_id)}
          >
            {tag?.name ?? "unknown"}
            <MinusCircle className="w-3 h-3" />
          </Badge>
        );
      })}
    </>
  );
};

export const ResourceTags = ({
  target,
  click_to_delete,
  className,
  disabled,
}: {
  target: TargetExcludingSystem;
  click_to_delete?: boolean;
  className?: string;
  disabled?: boolean;
}) => {
  const { toast } = useToast();
  const inv = useInvalidate();
  const { type, id } = target;
  const resource = useRead(`List${type}s`, {}).data?.find((d) => d.id === id);
  const { mutate } = useWrite("UpdateTagsOnResource", {
    onSuccess: () => {
      inv([`List${type}s`]);
      toast({ title: "Removed tag" });
    },
  });

  return (
    <TagsWithBadge
      tag_ids={resource?.tags}
      onBadgeClick={(tag_id) => {
        if (!click_to_delete) return;
        if (disabled) return;
        mutate({
          target,
          tags: resource!.tags.filter((tag) => tag !== tag_id),
        });
      }}
      className={className}
      icon={!disabled && click_to_delete && <MinusCircle className="w-3 h-3" />}
    />
  );
};

export const TagsWithBadge = ({
  tag_ids,
  onBadgeClick,
  className,
  icon,
}: {
  tag_ids?: string[];
  onBadgeClick?: (tag_id: string) => void;
  className?: string;
  icon?: ReactNode;
}) => {
  const all_tags = useRead("ListTags", {}).data;
  const get_tag = (tag_id: string) =>
    all_tags?.find((t) => t._id?.$oid === tag_id);
  return (
    <>
      {tag_ids?.map((tag_id) => {
        const tag = get_tag(tag_id);
        const color = tag_background_class(tag?.color);
        return (
          <Badge
            key={tag_id}
            variant="secondary"
            className={cn(
              "gap-2 px-1.5 py-0.5 cursor-pointer text-nowrap bg-opacity-30 hover:bg-opacity-70",
              color,
              `hover:${color}`,
              className
            )}
            onClick={() => onBadgeClick && onBadgeClick(tag_id)}
          >
            {tag?.name ?? "unknown"}
            {icon}
          </Badge>
        );
      })}
    </>
  );
};

export const TableTags = ({ tag_ids }: { tag_ids: string[] }) => {
  const { toggle_tag } = useTags();
  return (
    <div className="flex gap-1 flex-wrap">
      <TagsWithBadge tag_ids={tag_ids} onBadgeClick={toggle_tag} />
    </div>
  );
};

export const AddTags = ({ target }: { target: TargetExcludingSystem }) => {
  const { toast } = useToast();

  const { type, id } = target;
  const resource = useRead(`List${type}s`, {}).data?.find((d) => d.id === id);

  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");

  useShiftKeyListener("T", () => setOpen(true));

  const all_tags = useRead("ListTags", {}).data ?? [];
  const all_tag_names = all_tags.map((tag) => tag.name);

  const inv = useInvalidate();

  const { mutate: update } = useWrite("UpdateTagsOnResource", {
    onSuccess: () => {
      inv([`List${type}s`]);
      toast({ title: `Added tag ${search}` });
      setOpen(false);
    },
  });

  const { mutateAsync: create } = useWrite("CreateTag", {
    onSuccess: () => inv([`ListTags`]),
  });

  useEffect(() => {
    if (open) setSearch("");
  }, [open]);

  const create_tag = async () => {
    if (!search) return toast({ title: "Must provide tag name in input" });
    const tag = await create({ name: search });
    update({
      target,
      tags: [...(resource?.tags ?? []), tag._id!.$oid],
    });
    setOpen(false);
  };

  if (!resource) return null;

  const filtered = filterBySplit(all_tags, search, (item) => item.name)?.sort(
    (a, b) => {
      if (a.name > b.name) {
        return 1;
      } else if (a.name < b.name) {
        return -1;
      } else {
        return 0;
      }
    }
  );

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button variant="secondary" className="px-2 py-0.5 h-fit">
          <PlusCircle className="w-3" />
          {/* <Badge
            variant="outline"
            className="text-muted-foreground hidden md:inline-flex"
          >
            shift + t
          </Badge> */}
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[200px] p-0" sideOffset={12} align="start">
        <Command shouldFilter={false}>
          <CommandInput
            placeholder="Search / Create"
            className="h-9"
            value={search}
            onValueChange={setSearch}
          />
          <CommandList>
            <CommandEmpty className="m-1">
              <Button
                variant="ghost"
                onClick={create_tag}
                className="w-full flex items-center justify-between hover:bg-accent"
              >
                Create Tag
                <PlusCircle className="w-4" />
              </Button>
            </CommandEmpty>
            <CommandGroup>
              {filtered
                ?.filter((tag) => !resource?.tags.includes(tag._id!.$oid))
                .map((tag) => (
                  <CommandItem
                    key={tag._id?.$oid}
                    value={tag.name}
                    onSelect={() =>
                      update({
                        target,
                        tags: [...(resource?.tags ?? []), tag._id!.$oid],
                      })
                    }
                    className="cursor-pointer flex items-center justify-between gap-2"
                  >
                    <div className="p-1">{tag.name}</div>
                    <div
                      className={cn(
                        "w-[25px] h-[25px] rounded-sm",
                        tag_background_class(tag.color)
                      )}
                    />
                  </CommandItem>
                ))}
              {search && !all_tag_names.includes(search) && (
                <CommandItem onSelect={create_tag} className="cursor-pointer">
                  <div className="w-full p-1 flex items-center justify-between">
                    Create Tag
                    <PlusCircle className="w-4" />
                  </div>
                </CommandItem>
              )}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
};
