import { BoxProps, Flex, FlexProps, Group, Stack } from "@mantine/core";
import { Types } from "komodo_client";
import { Check, X } from "lucide-react";
import { fmtDate, fmtUpperCamelcase } from "@/lib/formatting";
import { ICONS } from "@/theme/icons";
import { useAlertDetails } from "./details";
import AlertLevel from "./level";
import ResourceLink from "@/resources/link";

export default function AlertCard({
  alert,
  smallHidden,
  accent,
  onClick: _onClick,
  large,
}: {
  alert: Types.Alert;
  smallHidden?: boolean;
  accent?: boolean;
  onClick?: () => void;
  large?: boolean;
}) {
  const { open: openDetails } = useAlertDetails();

  const FirstRow = (flexProps: FlexProps) => (
    <Flex
      justify="space-between"
      fz={{ base: "xs", md: "sm", lg: "md" }}
      {...flexProps}
    >
      <Group wrap="nowrap" gap="xs">
        <Icon alert={alert} />
        {fmtUpperCamelcase(alert.data.type)}
      </Group>
      <Group c="dimmed" wrap="nowrap" gap="xs">
        <ICONS.Calendar size="1rem" />
        {fmtDate(new Date(alert.ts))}
      </Group>
    </Flex>
  );

  const containerProps: BoxProps = {
    visibleFrom: smallHidden ? "xl" : undefined,
    style: {
      cursor: "pointer",
      borderBottom: "1px solid var(--mantine-color-accent-border-4)",
    },
    px: "lg",
    py: "sm",
    bg: accent ? "accent.0" : undefined,
  };

  const onClick = () => {
    openDetails(alert._id?.$oid!);
    _onClick?.();
  };

  if (large) {
    return (
      <Stack onClick={onClick} gap="0.4rem" {...containerProps}>
        <FirstRow />
        <Flex justify="space-between" c="dimmed">
          <Group gap="xs">
            {alert.target.type === "System" ? (
              <>
                <ICONS.System size="1rem" />
                System
              </>
            ) : (
              <ResourceLink
                onClick={_onClick}
                type={alert.target.type}
                id={alert.target.id}
              />
            )}
          </Group>
          <AlertLevel level={alert.level} />
        </Flex>
      </Stack>
    );
  } else {
    return <FirstRow onClick={onClick} {...containerProps} />;
  }
}

const Icon = ({ alert }: { alert: Types.Alert }) => {
  if (alert.resolved) {
    return <Check size="1rem" color="green" />;
  } else {
    return <X size="1rem" color="red" />;
  }
};
