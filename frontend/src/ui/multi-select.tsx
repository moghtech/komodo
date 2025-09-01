import * as React from "react";
import { Check, ChevronsUpDown, X } from "lucide-react";
import { cn } from "@/lib/utils";
import { Badge } from "@/ui/badge";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "@/ui/popover";
import { Skeleton } from "@/ui/skeleton";

interface MultiSelectProps {
  options?: { label: string; value: string }[];
  value: string[];
  onChange: (selected: string[]) => void;
  placeholder?: string;
  className?: string;
  isLoading?: boolean;
  disabled?: boolean;
}

function MultiSelect({
  options,
  value,
  onChange,
  placeholder = "Select items...",
  className,
  isLoading = false,
  disabled = false,
}: MultiSelectProps) {
  const [open, setOpen] = React.useState(false);

  const handleUnselect = (item: string) => {
    onChange(value.filter((i) => i !== item));
  };

  const handleSelect = (item: string) => {
    if (value.includes(item)) {
      handleUnselect(item);
    } else {
      onChange([...value, item]);
    }
  };

  return (
    <div className={cn("w-full", className)}>
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger
          className={cn(
            "flex h-full w-full transition-all items-center justify-between rounded-md border border-input bg-background text-sm",
            "focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
            "disabled:cursor-not-allowed disabled:opacity-50",
            "hover:bg-accent hover:text-accent-foreground"
          )}
          disabled={disabled}
          aria-expanded={open}
        >
          <div className="flex justify-between flex-1 overflow-hidden">
            <div
              className="flex gap-1 flex-1 py-2 px-3 overflow-x-auto"
              style={{
                scrollbarWidth: "thin",
                scrollbarColor: "hsl(var(--border)) transparent",
              }}
            >
              {value.length === 0 ? (
                <span className="text-muted-foreground truncate">
                  {placeholder}
                </span>
              ) : (
                value.map((item) => {
                  const option = options?.find((opt) => opt.value === item);
                  return (
                    <Badge key={item} variant="default" className="text-xs">
                      {option?.label}
                      <span
                        role="button"
                        tabIndex={0}
                        className="ml-1 hover:bg-destructive transition-all hover:text-destructive-foreground rounded-full p-0.5"
                        onKeyDown={(e) =>
                          e.key === "Enter" && handleUnselect(item)
                        }
                        onClick={() => handleUnselect(item)}
                      >
                        <X className="h-3 w-3" />
                      </span>
                    </Badge>
                  );
                })
              )}
            </div>
            <hr className="border-l border-border bg-red-300 h-6 mx-0.5 my-auto" />
            <span
              role="button"
              onClick={(e) => {
                e.stopPropagation();
                setOpen((prev) => !prev);
              }}
              tabIndex={0}
              className={cn(
                "p-1 mx-1.5 my-auto h-full outline-none",
                "focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
                "hover:bg-accent/50 rounded-sm cursor-pointer"
              )}
            >
              <ChevronsUpDown className="h-4 w-4 shrink-0 opacity-50" />
            </span>
          </div>
        </PopoverTrigger>
        <PopoverContent className="w-full p-0" align="start">
          <Command>
            <CommandInput autoFocus={false} placeholder="Search items..." />
            <CommandList>
              <CommandEmpty className="p-0">
                {isLoading ? (
                  <div className="p-2">
                    {Array.from({ length: 6 }).map((_, index) => (
                      <Skeleton
                        key={index}
                        className="h-4 w-full mb-1 last:mb-0"
                      />
                    ))}
                  </div>
                ) : (
                  <div className="text-center text-sm py-4 text-muted-foreground">
                    No items found.
                  </div>
                )}
              </CommandEmpty>
              <CommandGroup>
                {options?.map((option) => (
                  <CommandItem
                    key={option.value}
                    value={option.value}
                    onSelect={() => handleSelect(option.value)}
                  >
                    <Check
                      className={cn(
                        "mr-2 h-4 w-4",
                        value.includes(option.value)
                          ? "opacity-100"
                          : "opacity-0"
                      )}
                    />
                    {option.label}
                  </CommandItem>
                ))}
              </CommandGroup>
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>
    </div>
  );
}

export {MultiSelect}
