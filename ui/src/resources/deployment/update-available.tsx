import {
  useExecute,
  useInvalidate,
  usePermissions,
  useRead,
  useWrite,
} from "@/lib/hooks";
import { notifications } from "@mantine/notifications";
import { useDeployment } from ".";
import { Types } from "komodo_client";
import { ActionIcon, Box, Button, HoverCard } from "@mantine/core";
import { ICONS } from "@/theme/icons";
import ConfirmModalWithDisable from "@/components/confirm-modal-with-disable";
import { hexColorByIntention } from "@/lib/color";

export default function DeploymentUpdateAvailable({
  id,
  small,
}: {
  id: string;
  small?: boolean;
}) {
  const { canExecute } = usePermissions({ type: "Deployment", id });
  const { mutateAsync: deploy, isPending } = useExecute("Deploy");
  const inv = useInvalidate();
  const { mutate: checkForUpdate, isPending: checkPending } = useWrite(
    "CheckDeploymentForUpdate",
    {
      onSuccess: () => {
        notifications.show({ message: "Checked for updates", color: "blue" });
        inv(["ListDeployments"]);
      },
    },
  );
  const deploying = useRead(
    "GetDeploymentActionState",
    { deployment: id },
    { refetchInterval: 5_000 },
  ).data?.deploying;

  const pending = isPending || deploying;

  const deployment = useDeployment(id);
  const info = deployment?.info;
  const state = info?.state ?? Types.DeploymentState.Unknown;
  if (
    !info ||
    info.build_id ||
    [Types.DeploymentState.NotDeployed, Types.DeploymentState.Unknown].includes(
      state,
    )
  ) {
    return null;
  }

  if (small || !canExecute) {
    if (!info?.update_available) {
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
                Update Available
              </Button>
            )}
          </HoverCard.Target>
          <HoverCard.Dropdown>
            There is a newer image available.
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
          onClick={() => checkForUpdate({ deployment: id })}
          loading={checkPending}
        >
          Check
        </Button>
      </Box>
      {info?.update_available && (
        <Box>
          <ConfirmModalWithDisable
            title={
              <>
                Confirm <b>Redeploy</b>
              </>
            }
            confirmText={deployment.name}
            icon={<ICONS.UpdateAvailable size="1rem" />}
            targetProps={{
              variant: "outline",
              bd: "1px solid var(--mantine-color-blue-7)",
            }}
            onConfirm={() =>
              deploy({
                deployment: id,
              })
            }
            loading={pending}
          >
            Update Available
          </ConfirmModalWithDisable>
        </Box>
      )}
    </>
  );
}
