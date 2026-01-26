import { Flex, Group, Loader, Stack } from "@mantine/core";
import { Types } from "komodo_client";
import { UpdateStatus } from "komodo_client/dist/types";
import { Check, X } from "lucide-react";
import UpdateDetails from "./details";
import { fmtDate, fmtOperation, fmtVersion } from "@/lib/formatting";
import { versionIsNone } from "@/lib/utils";
import { ICONS } from "@/lib/icons";

export default function UpdateCard({
  update,
  smallHidden,
  accent,
}: {
  update: Types.UpdateListItem;
  smallHidden?: boolean;
  accent?: boolean;
}) {
  return (
    <UpdateDetails
      id={update.id}
      target={(open) => {
        return (
          <Flex
            visibleFrom={smallHidden ? "xl" : undefined}
            justify="space-between"
            style={{ cursor: "pointer" }}
            onClick={open}
            px="lg"
            py="sm"
            bg={accent ? "accent" : undefined}
          >
            <Stack>
              <Group>
                <Icon update={update} />
                {fmtOperation(update.operation)}
              </Group>
              {!versionIsNone(update.version) && (
                <Group c="dimmed">
                  <ICONS.Version size="1rem" />
                  {fmtVersion(update.version)}
                </Group>
              )}
            </Stack>
            <Stack>
              <Group c="dimmed">
                <ICONS.Calendar size="1rem" />
                {fmtDate(new Date(update.start_ts))}
              </Group>
              {/* <UpdateUser user_id={update.operator} muted /> */}
            </Stack>
          </Flex>
        );
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
