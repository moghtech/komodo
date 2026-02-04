import { useRead } from "@/lib/hooks";
import { Types } from "komodo_client";
import UpdateCard from "./card";
import { Button, ScrollArea, ScrollAreaProps, Stack } from "@mantine/core";
import { ICONS } from "@/theme/icons";
import { Link } from "react-router-dom";

export interface UpdateListProps extends ScrollAreaProps {
  query?: Types.ListUpdates["query"];
  max?: number;
  showAllLink?: string;
  onUpdateClick?: (update: Types.UpdateListItem) => void;
  large?: boolean;
}

export default function UpdateList({
  query,
  max,
  showAllLink,
  onUpdateClick,
  large,
  ...scrollProps
}: UpdateListProps) {
  const updates = useRead("ListUpdates", {
    query,
  }).data;
  return (
    <ScrollArea {...scrollProps}>
      <Stack pr="sm" gap="0">
        {updates?.updates.slice(0, max).map((update, i) => (
          <UpdateCard
            key={update.id}
            update={update}
            accent={i % 2 === 0}
            onClick={() => onUpdateClick?.(update)}
            large={large}
          />
        ))}
        {showAllLink && (
          <Button
            variant="light"
            c="inherit"
            leftSection={<ICONS.ExternalLink size="1rem" />}
            component={Link}
            to={showAllLink}
            fullWidth
          >
            Show All
          </Button>
        )}
      </Stack>
    </ScrollArea>
  );
}
