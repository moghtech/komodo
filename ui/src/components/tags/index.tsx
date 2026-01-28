import { Badge, BadgeProps, Loader } from "@mantine/core";
import { ReactNode } from "react";
import { useRead } from "@/lib/hooks";

export interface TagsProps extends BadgeProps {
  tag_ids?: string[];
  onBadgeClick?: (tag_id: string) => void;
  icon?: ReactNode;
  useName?: boolean;
}

export default function Tags({
  tag_ids,
  onBadgeClick,
  icon,
  useName,
  py = "0.3rem",
  ...badgeProps
}: TagsProps) {
  const allTags = useRead("ListTags", {}).data;
  const getTag = (tag: string) =>
    useName
      ? allTags?.find((t) => t.name === tag)
      : allTags?.find((t) => t._id?.$oid === tag);
  return (
    <>
      {tag_ids?.map((tag_id) => {
        const tag = getTag(tag_id);
        // const color = tagColor(tag?.color) + "60";
        return (
          <Badge
            key={tag_id}
            variant="filled"
            color={tag?.color ? `Tag${tag.color}.4` : "TagSlate.4"}
            c="inherit"
            onClick={() =>
              onBadgeClick &&
              (useName
                ? tag?.name && onBadgeClick(tag.name)
                : onBadgeClick(tag_id))
            }
            style={{ cursor: onBadgeClick ? "pointer" : undefined }}
            rightSection={icon}
            w="fit-content"
            h="fit-content"
            bdrs="sm"
            py={py}
            {...badgeProps}
          >
            {tag?.name ?? <Loader size="0.6rem" />}
          </Badge>
        );
      })}
    </>
  );
}
