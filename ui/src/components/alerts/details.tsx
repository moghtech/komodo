import {
  fmtDateWithMinutes,
  fmtDuration,
  fmtUpperCamelcase,
} from "@/lib/formatting";
import { useInvalidate, useRead, useUser, useWrite } from "@/lib/hooks";
import { ResourceComponents, UsableResource } from "@/resources";
import { Drawer, Group, Stack, Text } from "@mantine/core";
import { ICONS } from "@/theme/icons";
import { Clock, Link2 } from "lucide-react";
import CopyButton from "@/ui/copy-button";
import { MonacoEditor } from "@/components/monaco";
import LoadingScreen from "@/ui/loading-screen";
import { atom, useAtom } from "jotai";
import ResourceLink from "@/resources/link";
import { notifications } from "@mantine/notifications";
import ConfirmButton from "@/ui/confirm-button";

const alertDetailsAtom = atom<string>();

/** There is one alert details modal mounted, just change the target alert id */
export function useAlertDetails() {
  const [alertId, setAlertId] = useAtom(alertDetailsAtom);
  return {
    alertId,
    open: (alertId: string) => setAlertId(alertId),
    close: () => setAlertId(undefined),
  };
}

export default function AlertDetails() {
  const { alertId, close } = useAlertDetails();
  return (
    <Drawer
      opened={!!alertId}
      onClose={close}
      styles={{
        content: {
          flex: "none",
          width: 1400,
          maxWidth: "calc(100vw - 4rem)",
          height: "fit-content",
        },
      }}
      withCloseButton={false}
    >
      {alertId && <AlertDetailsContent id={alertId} close={close} />}
    </Drawer>
  );
}

export function AlertDetailsContent({
  id,
  close,
}: {
  id: string;
  close: () => void;
}) {
  const { data: alert } = useRead("GetAlert", { id });

  const isAdmin = useUser().data?.admin ?? false;
  const inv = useInvalidate();
  const { mutate: closeAlert, isPending: closePending } = useWrite(
    "CloseAlert",
    {
      onSuccess: () => {
        inv(["ListAlerts"], ["GetAlert"]);
        notifications.show({ message: "Closed alert." });
        close();
      },
    },
  );

  if (!alert) {
    return <LoadingScreen mt="0" h="50vh" />;
  }

  const Components =
    alert.target.type === "System"
      ? null
      : ResourceComponents[alert.target.type];

  return (
    <Stack gap="xl" m="md">
      {/** HEADER */}
      <Text fz="h1">{fmtUpperCamelcase(alert.data.type)}</Text>

      {/** DETAILS */}
      <Stack gap="sm">
        {/** RESOURCE / VERSION */}
        <Group gap="xs">
          {Components ? (
            <ResourceLink
              type={alert.target.type as UsableResource}
              id={alert.target.id}
              onClick={close}
            />
          ) : (
            <Group>
              <ICONS.Settings size="1rem" />
              System
            </Group>
          )}
        </Group>

        {/** DATE / DURATION / COPY LINK */}
        <Group>
          <Group>
            <ICONS.Calendar size="1rem" />
            {fmtDateWithMinutes(new Date(alert.ts))}
          </Group>
          <Group>
            <Clock size="1rem" />
            {alert.resolved_ts
              ? fmtDuration(alert.ts, alert.resolved_ts)
              : "ongoing"}
          </Group>
          <CopyButton
            content={`${location.origin}/alerts/${alert._id?.$oid}`}
            icon={<Link2 size="1rem" />}
            label="shareable link"
          />
        </Group>

        {isAdmin && !alert.resolved && (
          <ConfirmButton
            icon={<ICONS.Delete size="1rem" />}
            variant="destructive"
            onClick={() => closeAlert({ id: alert?._id?.$oid! })}
            loading={closePending}
          >
            Close Alert
          </ConfirmButton>
        )}
      </Stack>

      {/** Alert data */}
      <MonacoEditor
        value={JSON.stringify(alert.data.data, undefined, 2)}
        language="json"
        readOnly
      />
    </Stack>
  );
}
