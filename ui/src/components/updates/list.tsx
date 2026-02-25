import { useRead } from "@/lib/hooks";
import { Types } from "komodo_client";
import UpdateCard from "./card";
import { Button, Box, BoxProps, Stack, Center } from "@mantine/core";
import { ICONS } from "@/theme/icons";
import { Link } from "react-router-dom";

export interface UpdateListProps extends BoxProps {
  query?: Types.ListUpdates["query"];
  max?: number;
  showAllLink?: string;
  onUpdateClick?: (update: Types.UpdateListItem) => void;
  onClick?: () => void;
  large?: boolean;
}

export default function UpdateList({
  query,
  max,
  showAllLink,
  onUpdateClick,
  large,
  style,
  ...boxProps
}: UpdateListProps) {
  const updates = useRead(
    "ListUpdates",
    {
      query,
    },
    { enabled: !!query },
  ).data;
  return (
    <Box style={{ overflow: "auto", ...style }} {...boxProps}>
      <Stack pr="sm" gap="0">
        {!updates?.updates.length && <Center c="dimmed">No Updates</Center>}
        {updates?.updates.slice(0, max).map((update, i) => (
          <UpdateCard
            key={update.id}
            update={update}
            accent={i % 2 === 0}
            onClick={() => onUpdateClick?.(update)}
            large={large}
          />
        ))}
        {showAllLink && typeof updates?.next_page === "number" && (
          <Button
            leftSection={<ICONS.ExternalLink size="1rem" />}
            component={Link}
            to={showAllLink}
            fullWidth
          >
            Show All
          </Button>
        )}
      </Stack>
    </Box>
  );
}
