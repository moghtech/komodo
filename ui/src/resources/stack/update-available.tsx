import {
  useExecute,
  useInvalidate,
  usePermissions,
  useRead,
  useWrite,
} from "@/lib/hooks";
import { notifications } from "@mantine/notifications";
import { useFullStack, useStack } from ".";
import { Types } from "komodo_client";
import {
  Badge,
  Box,
  Button,
  Group,
  HoverCard,
  Stack,
  Text,
  ThemeIcon,
  VisuallyHidden,
} from "@mantine/core";
import { ICONS } from "@/lib/icons";
import ConfirmModalWithDisable from "@/components/confirm-modal-with-disable";
import { hexColorByIntention } from "mogh_ui";

export default function StackUpdateAvailable({
  id,
  small,
}: {
  id: string;
  small?: boolean;
}) {
  const { canExecute } = usePermissions({ type: "Stack", id });
  const { mutateAsync: deploy, isPending } = useExecute("DeployStack");
  const inv = useInvalidate();
  const { mutate: checkForUpdate, isPending: checkPending } = useWrite(
    "CheckStackForUpdate",
    {
      onSuccess: () => {
        notifications.show({ message: "Checked for updates", color: "blue" });
        inv(["ListStacks"]);
      },
    },
  );

  const deploying = useRead(
    "GetStackActionState",
    { stack: id },
    { refetchInterval: 5_000, enabled: !small && canExecute },
  ).data?.deploying;
  const pending = isPending || deploying;

  const stack = useStack(id);
  const fullStack = useFullStack(id);
  const info = stack?.info;
  const state = info?.state ?? Types.StackState.Unknown;

  if (
    !info ||
    [Types.StackState.Down, Types.StackState.Unknown].includes(state)
  ) {
    return null;
  }

  const servicesWithUpdate =
    info?.services.filter((s) => s.update_available) ?? [];

  const updateAvailable = servicesWithUpdate.length > 0;

  if (!canExecute) {
    if (!updateAvailable) {
      return null;
    }
    return (
      <Box>
        <HoverCard>
          <HoverCard.Target>
            {small ? (
              <ThemeIcon
                aria-label="Update available"
                variant="outline"
                bd={"1px solid " + hexColorByIntention("Neutral")}
                size="md"
              >
                <ICONS.UpdateAvailable size="1rem" />
              </ThemeIcon>
            ) : (
              <Badge
                variant="outline"
                bd={"1px solid " + hexColorByIntention("Neutral")}
                leftSection={<ICONS.UpdateAvailable size="1rem" />}
                size="lg"
                tt="none"
              >
                Update
                {(info?.services.filter((s) => s.update_available).length ??
                  0) > 1
                  ? "s"
                  : ""}{" "}
                Available
              </Badge>
            )}
          </HoverCard.Target>
          <HoverCard.Dropdown>
            <Services
              services={info?.services}
              latestServices={fullStack?.info?.latest_services}
            />
          </HoverCard.Dropdown>
        </HoverCard>
      </Box>
    );
  }

  const updateAction = (
    <ConfirmModalWithDisable
      title={
        <>
          Confirm <b>Redeploy</b>
        </>
      }
      confirmText={stack.name}
      confirmButtonContent={small ? "Update Available" : undefined}
      icon={small ? undefined : <ICONS.UpdateAvailable size="1rem" />}
      targetNoIcon={small}
      targetProps={
        small
          ? {
              variant: "outline",
              bd: "1px solid var(--mantine-color-blue-7)",
              w: "auto",
              miw: "auto",
              px: "xs",
            }
          : {
              variant: "outline",
              bd: "1px solid var(--mantine-color-blue-7)",
            }
      }
      onConfirm={() =>
        deploy({
          stack: id,
          services: fullStack?.config?.auto_update_all_services
            ? []
            : servicesWithUpdate.map((s) => s.service),
        })
      }
      loading={pending}
      topAdditonal={
        !fullStack?.config?.auto_update_all_services && (
          <Stack className="bordered-light" p="md" bdrs="md" gap="sm">
            <Text size="lg">
              Service
              {servicesWithUpdate.length === 1 ? "" : "s"} with update:
            </Text>
            <Services
              services={info?.services}
              latestServices={fullStack?.info?.latest_services}
            />
          </Stack>
        )
      }
    >
      {small ? (
        <>
          <VisuallyHidden>Redeploy stack update</VisuallyHidden>
          <ICONS.UpdateAvailable size="1rem" />
        </>
      ) : (
        "Update Available"
      )}
    </ConfirmModalWithDisable>
  );

  if (small) {
    return updateAvailable ? <Box>{updateAction}</Box> : null;
  }

  return (
    <>
      {updateAvailable && <Box>{updateAction}</Box>}
      <Box>
        <Button
          title="Check for updates"
          variant="outline"
          c="dimmed"
          rightSection={<ICONS.UpdateAvailable size="1rem" />}
          onClick={() => checkForUpdate({ stack: id })}
          loading={checkPending}
        >
          Check
        </Button>
      </Box>
    </>
  );
}

function Services({
  services,
  latestServices,
}: {
  services: Types.StackServiceWithUpdate[] | undefined;
  latestServices: Types.StackServiceNames[] | undefined;
}) {
  return (
    <Stack gap="0">
      {services
        ?.filter((service) => service.update_available)
        ?.map((s) => (
          <Group key={s.service} gap="xs">
            <Text c="dimmed">{s.service}</Text>
            <Text c="dimmed"> - </Text>
            <Text>
              {latestServices?.find((ser) => ser.service_name == s.service)
                ?.image ?? s.image}
            </Text>
          </Group>
        ))}
    </Stack>
  );
}
