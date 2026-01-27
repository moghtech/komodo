import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { Group, MantineStyleProps, Text } from "@mantine/core";

export interface UserAvatarProps {
  userId: string;
  iconSize?: string | number;
  dimmed?: boolean;
  onlyAvatar?: boolean;
  forceDefaultAvatar?: boolean;
  fz?: MantineStyleProps["fz"];
}

export default function UserAvatar({
  userId,
  iconSize = "1rem",
  dimmed,
  onlyAvatar,
  forceDefaultAvatar,
  fz = "md",
}: UserAvatarProps) {
  const user = useRead("GetUsername", { user_id: userId }).data;

  const avatar =
    forceDefaultAvatar || !user?.avatar ? (
      <ICONS.User
        size={iconSize}
        color={dimmed ? "var(--mantine-color-dimmed-0)" : undefined}
      />
    ) : (
      <img
        src={user.avatar}
        alt="avatar"
        style={{ width: iconSize, height: iconSize }}
        color={dimmed ? "var(--mantine-color-dimmed-0)" : undefined}
      />
    );

  if (onlyAvatar) {
    return avatar;
  }

  return (
    <Group>
      {avatar}
      <Text fz={fz}>{user?.username ?? "Unknown"}</Text>
    </Group>
  );
}
