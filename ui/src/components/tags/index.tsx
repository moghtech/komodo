import { Badge, Loader } from "@mantine/core";
import { ReactNode } from "react";
import { useRead } from "@/lib/hooks";
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
            className={
              className ? classes["tag"] + " " + className : classes["tag"]
            }
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
            bdrs="sm"
          >
            {tag?.name ?? <Loader size="0.6rem" />}
          </Badge>
        );
      })}
    </>
  );
}
