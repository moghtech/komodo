import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import {
  Group,
  GroupProps,
  MantineStyleProps,
  Text,
  TextProps,
} from "@mantine/core";

export interface UserAvatarProps extends TextProps {
  userId: string;
  iconSize?: string | number;
  dimmed?: boolean;
  onlyAvatar?: boolean;
  forceDefaultAvatar?: boolean;
  groupProps?: GroupProps;
}

export default function UserAvatar({
  userId,
  iconSize = "1rem",
  dimmed,
  onlyAvatar,
  forceDefaultAvatar,
  groupProps,
  ...textProps
}: UserAvatarProps) {
  const user = useRead("GetUsername", { user_id: userId }).data;

  const avatar =
    forceDefaultAvatar || !user?.avatar ? (
      <ICONS.User
        size={iconSize}
        color={dimmed ? "var(--mantine-color-dimmed-7)" : undefined}
      />
    ) : (
      <img
        src={user.avatar}
        alt="avatar"
        style={{ width: iconSize, height: iconSize }}
        color={dimmed ? "var(--mantine-color-dimmed-7)" : undefined}
      />
    );

  if (onlyAvatar) {
    return avatar;
  }

  return (
    <Group gap="xs" {...groupProps}>
      {avatar}
      <Text fz={{ base: "xs", md: "sm", lg: "md" }} {...textProps}>
        {user?.username ?? "Unknown"}
      </Text>
    </Group>
  );
}
