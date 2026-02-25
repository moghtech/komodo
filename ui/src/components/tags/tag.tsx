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
}
