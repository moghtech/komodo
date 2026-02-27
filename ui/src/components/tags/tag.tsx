import { Badge, BadgeProps, Loader } from "@mantine/core";
import { Types } from "komodo_client";
import { ReactNode } from "react";

export interface TagProps extends BadgeProps {
  tag: Types.Tag | undefined;
  icon?: ReactNode;
  onClick?: () => void;
}

export default function Tag({
  tag,
  icon,
  onClick,
  py = "0.3rem",
  ...badgeProps
}: TagProps) {
  return (
    <Badge
      variant="filled"
      color={tag?.color ? `Tag${tag.color}.4` : "TagSlate.4"}
      onClick={onClick}
      style={{ cursor: onClick ? "pointer" : undefined }}
      rightSection={icon}
      className="text-ellipsis"
      w="fit-content"
      h="fit-content"
      bdrs="sm"
      py={py}
      tt="none"
      fz={{ base: "xs", lg: "sm" }}
      styles={{
        label: { width: "fit-content", height: "fit-content" },
      }}
      {...badgeProps}
    >
      {tag?.name ?? <Loader size="0.6rem" />}
    </Badge>
  );
}
