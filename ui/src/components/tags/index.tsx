import { Badge } from "@mantine/core";
import { ReactNode } from "react";
import { useRead } from "@/lib/hooks";
import { tagColor } from "@/lib/color";
import classes from "./index.module.scss";

export default function Tags({
  tag_ids,
  onBadgeClick,
  className,
  icon,
  useName,
}: {
  tag_ids?: string[];
  onBadgeClick?: (tag_id: string) => void;
  className?: string;
  icon?: ReactNode;
  useName?: boolean;
}) {
  const all_tags = useRead("ListTags", {}).data;
  const get_tag = (tag: string) =>
    useName
      ? all_tags?.find((t) => t.name === tag)
      : all_tags?.find((t) => t._id?.$oid === tag);
  return (
    <>
      {tag_ids?.map((tag_id) => {
        const tag = get_tag(tag_id);
        const color = tagColor(tag?.color);
        return (
          <Badge
            key={tag_id}
            variant="filled"
            className={
              className ? classes["tag"] + " " + className : classes["tag"]
            }
            color={color}
            onClick={() =>
              onBadgeClick &&
              (useName
                ? tag?.name && onBadgeClick(tag.name)
                : onBadgeClick(tag_id))
            }
            style={{ cursor: onBadgeClick ? "pointer" : undefined }}
            rightSection={icon}
            w="fit-content"
          >
            {tag?.name ?? "unknown"}
          </Badge>
        );
      })}
    </>
  );
}
