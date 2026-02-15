import { BadgeProps } from "@mantine/core";
import { ReactNode, useCallback } from "react";
import { useRead } from "@/lib/hooks";
import Tag from "./tag";

export interface TagsProps extends BadgeProps {
  tagIds?: string[];
  onBadgeClick?: (tag_id: string) => void;
  icon?: ReactNode;
  useName?: boolean;
}

export default function Tags({
  tagIds,
  onBadgeClick,
  icon,
  useName,
  ...badgeProps
}: TagsProps) {
  const allTags = useRead("ListTags", {}).data;
  const getTag = useCallback(
    (tag: string) =>
      useName
        ? allTags?.find((t) => t.name === tag)
        : allTags?.find((t) => t._id?.$oid === tag),
    [allTags, useName],
  );
  return (
    <>
      {tagIds?.map((tagId) => {
        const tag = getTag(tagId);
        return (
          <Tag
            key={tagId}
            tag={tag}
            icon={icon}
            onClick={() =>
              onBadgeClick &&
              (useName
                ? tag?.name && onBadgeClick(tag.name)
                : onBadgeClick(tagId))
            }
            {...badgeProps}
          />
        );
      })}
    </>
  );
}
