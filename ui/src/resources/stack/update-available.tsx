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
  ActionIcon,
  Box,
  Button,
  Group,
  HoverCard,
  Stack,
  Text,
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
    { refetchInterval: 5000 },
  ).data?.deploying;
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

  const pending = isPending || deploying;

  // No quick deploy action / check for update button
  if (small || !canExecute) {
    if (!updateAvailable) {
      return null;
    }
    return (
      <Box>
        <HoverCard>
          <HoverCard.Target>
            {small ? (
              <ActionIcon
                variant="outline"
                bd={"1px solid " + hexColorByIntention("Neutral")}
                size="md"
              >
                <ICONS.UpdateAvailable size="1rem" />
              </ActionIcon>
            ) : (
              <Button
                variant="outline"
                bd={"1px solid " + hexColorByIntention("Neutral")}
                leftSection={<ICONS.UpdateAvailable size="1rem" />}
              >
                Update
                {(info?.services.filter((s) => s.update_available).length ??
                  0) > 1
                  ? "s"
                  : ""}{" "}
                Available
              </Button>
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

  return (
    <>
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
      {updateAvailable && (
        <Box>
          <ConfirmModalWithDisable
            title={
              <>
                Confirm <b>Redeploy</b>
              </>
            }
            confirmText={stack.name}
            icon={<ICONS.UpdateAvailable size="1rem" />}
            targetProps={{
              variant: "outline",
              bd: "1px solid var(--mantine-color-blue-7)",
            }}
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
            Update Available
          </ConfirmModalWithDisable>
        </Box>
      )}
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
