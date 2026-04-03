import {
  fmtDateWithMinutes,
  fmtDuration,
  fmtUpperCamelcase,
} from "@/lib/formatting";
import { useInvalidate, useRead, useUser, useWrite } from "@/lib/hooks";
import { ResourceComponents, UsableResource } from "@/resources";
import { ActionIcon, Drawer, Group, Stack, Text } from "@mantine/core";
import { ICONS } from "@/lib/icons";
import { Clock, Link2 } from "lucide-react";
import { CopyButton } from "mogh_ui";
import { MonacoEditor } from "@/components/monaco";
import LoadingScreen from "@/ui/loading-screen";
import { atom, useAtom } from "jotai";
import ResourceLink from "@/resources/link";
import { notifications } from "@mantine/notifications";
import { ConfirmButton } from "mogh_ui";
import { To, useLocation, useNavigate } from "react-router-dom";

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

export default function AlertDetails({
  alertId: __alertId,
}: {
  alertId?: string;
}) {
  const { alertId: _alertId, close } = useAlertDetails();
  const alertId = __alertId ?? _alertId;
  // https://github.com/remix-run/react-router/discussions/9788#discussioncomment-4604278
  const navTo = (useLocation().key === "default" ? "/" : -1) as To;
  const nav = useNavigate();
  return (
    <Drawer
      opened={!!alertId}
      onClose={__alertId ? () => nav(navTo) : close}
      styles={{
        content: {
          flex: "none",
          width: 1400,
          maxWidth: "calc(100vw - 2rem)",
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
  const { data: alert } = useRead(
    "GetAlert",
    { id },
    { refetchInterval: 10_000 },
  );

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

  const RC =
    alert.target.type === "System"
      ? null
      : ResourceComponents[alert.target.type];

  return (
    <Stack gap="xl" m="md">
      {/** HEADER */}
      <Group justify="space-between">
        <Text fz="h1">{fmtUpperCamelcase(alert.data.type)}</Text>
        <ActionIcon size="lg" variant="filled" color="red" onClick={close}>
          <ICONS.Clear size="1.3rem" />
        </ActionIcon>
      </Group>

      {/** DETAILS */}
      <Stack gap="sm">
        {/** RESOURCE / VERSION */}
        <Group gap="xs">
          {RC ? (
            <ResourceLink
              type={alert.target.type as UsableResource}
              id={alert.target.id}
              onClick={close}
              fz="md"
            />
          ) : (
            <Group gap="xs" wrap="nowrap">
              <ICONS.Settings size="1rem" />
              System
            </Group>
          )}
        </Group>

        {/** DATE / DURATION / COPY LINK */}
        <Group>
          <Group gap="xs" wrap="nowrap">
            <ICONS.Calendar size="1rem" />
            {fmtDateWithMinutes(new Date(alert.ts))}
          </Group>
          <Group gap="xs" wrap="nowrap">
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
            onClick={() => closeAlert({ id: alert?._id?.$oid! })}
            loading={closePending}
            confirmProps={{ variant: "filled", color: "red" }}
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
