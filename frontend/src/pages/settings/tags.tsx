import { ConfirmButton } from "@components/util";
import {
  useInvalidate,
  useRead,
  useSetTitle,
  useUser,
  useWrite,
} from "@lib/hooks";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@ui/dialog";
import { Button } from "@ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@ui/card";
import { useToast } from "@ui/use-toast";
import {
  Trash,
  PlusCircle,
  Loader2,
  Check,
  Search,
  SearchX,
} from "lucide-react";
import { useState } from "react";
import { Input } from "@ui/input";
import { UpdateUser } from "@components/updates/details";
import { DataTable } from "@ui/data-table";
import { Types } from "komodo_client";
import { Popover, PopoverContent, PopoverTrigger } from "@ui/popover";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@ui/command";
import { cn, filterBySplit } from "@lib/utils";
import { fmt_upper_camelcase } from "@lib/formatting";
import { tag_background_class } from "@lib/color";

export const Tags = () => {
  useSetTitle("Tags");
  const user = useUser().data!;

  const [search, setSearch] = useState("");

  const tags = useRead("ListTags", {}).data;

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <CreateTag />
        <div className="relative">
          <Search className="w-4 absolute top-[50%] left-3 -translate-y-[50%] text-muted-foreground" />
          <Input
            placeholder="search..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-8 w-[200px] lg:w-[300px]"
          />
        </div>
      </div>
      <DataTable
        tableKey="tags"
        data={tags?.filter((tag) => tag.name.includes(search)) ?? []}
        columns={[
          {
            header: "Name",
            size: 200,
            accessorKey: "name",
          },
          {
            header: "Color",
            size: 200,
            cell: ({ row }) => (
              <ColorSelector
                tag_id={row.original._id?.$oid!}
                color={row.original.color!}
                disabled={!user.admin && row.original.owner !== user._id?.$oid}
              />
            ),
          },
          {
            header: "Owner",
            size: 200,
            cell: ({ row }) =>
              row.original.owner ? (
                <UpdateUser user_id={row.original.owner} />
              ) : (
                "Unknown"
              ),
          },
          {
            header: "Delete",
            size: 200,
            cell: ({ row }) => (
              <DeleteTag
                tag_id={row.original._id!.$oid}
                disabled={!user.admin && row.original.owner !== user._id?.$oid}
              />
            ),
          },
        ]}
      />
    </div>
  );
};

export const TagCards = () => {
  const tags = useRead("ListTags", {}).data;
  const user = useUser().data!;
  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      {tags?.map((tag) => (
        <Card
          id={tag._id!.$oid}
          className="h-full hover:bg-accent/50 group-focus:bg-accent/50 transition-colors"
        >
          <CardHeader className="flex-row justify-between items-center">
            <CardTitle>{tag.name}</CardTitle>
            <DeleteTag
              tag_id={tag._id!.$oid}
              disabled={!user.admin && tag.owner !== user._id?.$oid}
            />
          </CardHeader>
          <CardContent className="text-sm text-muted-foreground">
            {tag.owner && (
              <div>
                owner: <UpdateUser user_id={tag.owner} />
              </div>
            )}
          </CardContent>
        </Card>
      ))}
    </div>
  );
};

export const CreateTag = () => {
  const { toast } = useToast();
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const invalidate = useInvalidate();
  const { mutate, isPending } = useWrite("CreateTag", {
    onSuccess: () => {
      invalidate(["ListTags"]);
      toast({ title: "Tag Created" });
      setOpen(false);
    },
  });
  const submit = () => mutate({ name });
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant="secondary" className="items-center gap-2">
          New Tag <PlusCircle className="w-4 h-4" />
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create Tag</DialogTitle>
        </DialogHeader>
        <div className="py-8 flex flex-col gap-4">
          <div className="flex items-center justify-between">
            Name
            <Input
              className="w-72"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </div>
        </div>
        <DialogFooter className="flex justify-end">
          <Button className="gap-4" onClick={submit} disabled={isPending}>
            Submit
            {isPending ? (
              <Loader2 className="w-4 animate-spin" />
            ) : (
              <Check className="w-4" />
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

const DeleteTag = ({
  tag_id,
  disabled,
}: {
  tag_id: string;
  disabled: boolean;
}) => {
  const invalidate = useInvalidate();
  const { toast } = useToast();
  const { mutate, isPending } = useWrite("DeleteTag", {
    onSuccess: () => {
      invalidate(["ListTags"]);
      toast({ title: "Tag Deleted" });
    },
  });
  return (
    <ConfirmButton
      title="Delete"
      icon={<Trash className="w-4 h-4" />}
      onClick={() => mutate({ id: tag_id })}
      disabled={disabled}
      loading={isPending}
    />
  );
};

const ColorSelector = ({
  tag_id,
  color,
  disabled,
}: {
  tag_id: string;
  color: Types.TagColor;
  disabled: boolean;
}) => {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [loadingColor, setLoadingColor] = useState<Types.TagColor>();
  const { mutateAsync } = useWrite("UpdateTagColor");
  const inv = useInvalidate();
  const onSelect = async (color: Types.TagColor) => {
    setLoadingColor(color);
    await mutateAsync({ tag: tag_id, color });
    inv(["ListTags"]);
    setLoadingColor(undefined);
    setOpen(false);
  };
  const filtered = filterBySplit(
    Object.values(Types.TagColor),
    search,
    (item) => item
  );
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="secondary"
          className="flex justify-between gap-2 w-[160px]"
          disabled={disabled}
        >
          {fmt_upper_camelcase(color) || "Select Color"}
          <div
            className={cn(
              "w-[25px] h-[25px] rounded-sm bg-opacity-70",
              tag_background_class(color)
            )}
          />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[200px] max-h-[300px] p-0" align="end">
        <Command shouldFilter={false}>
          <CommandInput
            placeholder="Search Colors"
            className="h-9"
            value={search}
            onValueChange={setSearch}
          />
          <CommandList>
            <CommandEmpty className="flex justify-evenly items-center pt-3 pb-2">
              {"No Colors Found"}
              <SearchX className="w-3 h-3" />
            </CommandEmpty>

            <CommandGroup>
              {filtered.map((color) => (
                <CommandItem
                  key={color}
                  onSelect={() => onSelect(color)}
                  className="flex items-center justify-between gap-2 cursor-pointer"
                >
                  {color !== loadingColor && (
                    <div className="p-1">{fmt_upper_camelcase(color)}</div>
                  )}
                  {color === loadingColor && (
                    <Loader2 className="w-4 h-4 animate-spin mx-1" />
                  )}
                  <div
                    className={cn(
                      "w-[25px] h-[25px] rounded-sm bg-opacity-70",
                      tag_background_class(color)
                    )}
                  />
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
};
