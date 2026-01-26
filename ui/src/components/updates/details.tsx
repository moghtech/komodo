import { fmtOperation, fmtVersion } from "@/lib/formatting";
import { useRead } from "@/lib/hooks";
import { useWebsocketMessages } from "@/lib/socket";
import { versionIsNone } from "@/lib/utils";
import { ResourceComponents } from "@/resources";
import { Center, Drawer, Loader, Stack, Text } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { ReactNode, useEffect } from "react";

export default function UpdateDetails({
  id,
  target,
}: {
  id: string;
  target: (open: () => void) => ReactNode;
}) {
  const [opened, { open, close }] = useDisclosure(false);
  const Target = target(open);
  return (
    <>
      {Target}
      <UpdateDetailsInner id={id} open={opened} close={close} />
    </>
  );
}

export function UpdateDetailsInner({
  id,
  open,
  close,
}: {
  id: string;
  open: boolean;
  close: () => void;
}) {
  return (
    <Drawer opened={open} onClose={close} position="top" size="lg">
      <UpdateDetailsContent id={id} close={close} />
    </Drawer>
  );
}

export function UpdateDetailsContent({
  id,
  open,
  close,
}: {
  id: string;
  open?: boolean;
  close: () => void;
}) {
  const { data: update, refetch } = useRead(
    "GetUpdate",
    { id },
    { enabled: false },
  );
  useEffect(() => {
    // handle open state change loading
    if (open) {
      refetch();
    }
  }, [open]);
  // Since auto refetching is disabled, listen for updates on the update id and refetch
  useWebsocketMessages("update-details", (update) => {
    if (update.id === id) refetch();
  });
  if (!update) {
    return (
      <Center>
        <Loader size="xl" />
      </Center>
    );
  }
  const Components =
    update.target.type === "System"
      ? null
      : ResourceComponents[update.target.type];

  return (
    <Stack gap="lg">
      <Text fz="h1">
        {fmtOperation(update.operation)}{" "}
        {!versionIsNone(update.version) && fmtVersion(update.version)}
      </Text>
      <Stack gap="sm">
        {/* <UpdateUser user_id={update.operator} /> */}
      </Stack>
    </Stack>
  );
}
