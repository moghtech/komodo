import { languageFromPath, MonacoEditor } from "@/components/monaco";
import { useExecute, usePermissions, useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import ConfirmUpdate from "@/ui/config/confirm";
import UnsavedChanges from "@/ui/config/unsaved-changes";
import Section, { SectionProps } from "@/ui/section";
import { Button, Group } from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { notifications } from "@mantine/notifications";

export interface SwarmSecretEditProps extends SectionProps {
  swarm: string;
  secret: string;
}

export default function SwarmSecretEditSection({
  swarm,
  secret,
  ...sectionProps
}: SwarmSecretEditProps) {
  const [{ edit }, setEdit] = useLocalStorage<{ edit: string | undefined }>({
    key: `swarm-${swarm}-secret-${secret}-edit-v2`,
    defaultValue: { edit: undefined },
  });
  const {
    data: inspect,
    isPending,
    refetch,
    isError,
  } = useRead("InspectSwarmSecret", {
    swarm,
    secret,
  });
  const { canExecute } = usePermissions({
    type: "Swarm",
    id: swarm,
  });
  const { mutateAsync: confirmEdit } = useExecute("RotateSwarmSecret", {
    onSuccess: (res) => {
      notifications.show({
        message: res.success ? "Secret updated." : "Failed to update secret.",
        color: res.success ? "green" : "red",
      });
      setEdit({ edit: undefined });
      refetch();
    },
  });

  const name = inspect?.Spec?.Name;
  const language = name ? languageFromPath(name) : undefined;

  return (
    <Section
      isPending={isPending}
      error={
        isError
          ? "Failed to inspect swarm secret."
          : !secret
            ? `No swarm secret found with given name: ${secret}`
            : undefined
      }
      actions={
        canExecute && (
          <Group>
            {edit !== undefined && <UnsavedChanges />}{" "}
            <Button
              variant="outline"
              onClick={(e) => {
                e.stopPropagation();
                setEdit({ edit: undefined });
              }}
              disabled={edit === undefined}
              leftSection={<ICONS.History size="1rem" />}
              w={100}
            >
              Reset
            </Button>
            <ConfirmUpdate
              original={{ contents: "" }}
              update={{ contents: edit }}
              onConfirm={async () =>
                name &&
                edit !== undefined &&
                (await confirmEdit({ swarm, secret: name, data: edit }))
              }
              disabled={edit === undefined}
              language={language}
              loading={isPending}
            />
          </Group>
        )
      }
      {...sectionProps}
    >
      <MonacoEditor
        value={edit ?? "# Enter new secret and save to rotate"}
        language={language}
        onValueChange={(edit) => setEdit({ edit })}
        readOnly={!canExecute}
      />
    </Section>
  );
}
