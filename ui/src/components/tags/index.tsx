import { Badge, BadgeProps, Loader } from "@mantine/core";
import { ReactNode, useCallback } from "react";
import { useRead } from "@/lib/hooks";

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
  py = "0.3rem",
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
        // const color = tagColor(tag?.color) + "60";
        return (
          <Badge
            key={tagId}
            variant="filled"
            color={tag?.color ? `Tag${tag.color}.4` : "TagSlate.4"}
            onClick={() =>
              onBadgeClick &&
              (useName
                ? tag?.name && onBadgeClick(tag.name)
                : onBadgeClick(tagId))
            }
            style={{ cursor: onBadgeClick ? "pointer" : undefined }}
            rightSection={icon}
            w="fit-content"
            h="fit-content"
            bdrs="sm"
            py={py}
            tt="none"
            fz={{ base: "xs", lg: "sm" }}
            {...badgeProps}
          >
            {tag?.name ?? <Loader size="0.6rem" />}
          </Badge>
        );
      })}
    </>
  );
}
