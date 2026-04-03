import { useRead } from "@/lib/hooks";
import { Types } from "komodo_client";
import AlertCard from "./card";
import { Button, Box, BoxProps, Stack } from "@mantine/core";
import { ICONS } from "@/lib/icons";
import { Link } from "react-router-dom";

export interface AlertListProps extends BoxProps {
  query?: Types.ListAlerts["query"];
  max?: number;
  showAllLink?: string;
  onAlertClick?: (update: Types.Alert) => void;
  onClick?: () => void;
  large?: boolean;
}

export default function AlertList({
  query,
  max,
  showAllLink,
  onAlertClick,
  large,
  style,
  ...scrollProps
}: AlertListProps) {
  const alerts = useRead("ListAlerts", {
    query,
  }).data;
  return (
    <Box style={{ overflow: "auto", ...style }} {...scrollProps}>
      <Stack pr="sm" gap="0">
        {alerts?.alerts.slice(0, max).map((alert, i) => (
          <AlertCard
            key={alert._id?.$oid}
            alert={alert}
            accent={i % 2 === 0}
            onClick={() => onAlertClick?.(alert)}
            large={large}
          />
        ))}
        {showAllLink && (
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
