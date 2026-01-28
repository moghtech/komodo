import { BoxProps, Flex, FlexProps, Group, Loader, Stack } from "@mantine/core";
import { Types } from "komodo_client";
import { UpdateStatus } from "komodo_client/dist/types";
import { Check, X } from "lucide-react";
import UpdateDetails from "./details";
import { fmtDate, fmtOperation, fmtVersion } from "@/lib/formatting";
import { versionIsNone } from "@/lib/utils";
import { ICONS } from "@/lib/icons";
import { ResourceNameSimple } from "@/resources/common";
import UserAvatar from "@/components/user-avatar";

export default function UpdateCard({
  update,
  smallHidden,
  accent,
  onClick,
  large,
}: {
  update: Types.UpdateListItem;
  smallHidden?: boolean;
  accent?: boolean;
  onClick?: () => void;
  large?: boolean;
}) {
  const TargetIcon = ICONS[update.target.type];
  const FirstRow = (flexProps: FlexProps) => (
    <Flex justify="space-between" {...flexProps}>
      <Group>
        <Icon update={update} />
        {fmtOperation(update.operation)}
        {!versionIsNone(update.version) && (
          <Group c="dimmed" gap="xs">
            <ICONS.Version size="1rem" />
            {fmtVersion(update.version)}
          </Group>
        )}
      </Group>
      <Group c="dimmed">
        <ICONS.Calendar size="1rem" />
        {fmtDate(new Date(update.start_ts))}
      </Group>
    </Flex>
  );
  const containerProps: BoxProps = {
    visibleFrom: smallHidden ? "xl" : undefined,
    style: { cursor: "pointer" },
    px: "lg",
    py: "sm",
    bg: accent ? "accent.0" : undefined,
  };
  return (
    <UpdateDetails
      id={update.id}
      target={(open) => {
        if (large) {
          return (
            <Stack
              onClick={() => {
                open();
                onClick?.();
              }}
              gap="0"
              {...containerProps}
            >
              <FirstRow />
              <Flex justify="space-between" c="dimmed">
                <Group gap="xs">
                  {TargetIcon ? (
                    <TargetIcon size="1rem" />
                  ) : (
                    <ICONS.Unknown size="1rem" />
                  )}
                  {update.target.type === "System" ? (
                    "System"
                  ) : (
                    <ResourceNameSimple
                      type={update.target.type}
                      id={update.target.id}
                    />
                  )}
                </Group>
                <UserAvatar userId={update.operator} />
              </Flex>
            </Stack>
          );
        } else {
          return (
            <FirstRow
              onClick={() => {
                open();
                onClick?.();
              }}
              {...containerProps}
            />
          );
        }
      }}
    />
  );
}

const Icon = ({ update }: { update: Types.UpdateListItem }) => {
  if (update.status === UpdateStatus.Complete) {
    if (update.success) return <Check size="1rem" color="green" />;
    else return <X size="1rem" color="red" />;
  } else return <Loader size="1rem" />;
};
