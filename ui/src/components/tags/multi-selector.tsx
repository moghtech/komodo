import { useRead } from "@/lib/hooks";
import { ComboboxProps, Group } from "@mantine/core";
import TagSelector from "./selector";
import Tags from ".";
import { CircleMinus } from "lucide-react";

export interface TagMultiSelectorProps extends ComboboxProps {
  title?: string;
  /** The selected tag ids (or names with `useName`). */
  value: string[];
  onChange: (tags: string[]) => void;
  /** Use tag names as values instead of ids. */
  useName?: boolean;
  canCreate?: boolean;
}

export default function TagMultiSelector({
  title = "Select Tags",
  value,
  onChange,
  useName,
  canCreate,
  disabled,
  position = "bottom-start",
  ...props
}: TagMultiSelectorProps) {
  const tags = useRead("ListTags", {}).data;
  const otherTags = tags?.filter(
    (tag) => !value.includes(useName ? tag.name : tag._id?.$oid!),
  );

  return (
    <Group>
      <TagSelector
        title={title}
        tags={otherTags}
        onSelect={(tag) => onChange([...value, tag])}
        disabled={disabled}
        position={position}
        useName={useName}
        canCreate={canCreate}
        {...props}
      />

      <Tags
        tagIds={
          tags
            ?.filter((tag) => value.includes(tag.name))
            .map((tag) => tag.name) ?? []
        }
        onBadgeClick={
          disabled
            ? undefined
            : (toRemove) =>
                onChange(value.filter((tagName) => tagName !== toRemove))
        }
        icon={<CircleMinus size="1rem" />}
        fz="1rem"
        useName={useName}
      />
    </Group>
  );
}
