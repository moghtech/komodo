import { languageFromPath, MonacoEditor } from "@/components/monaco";
import { useExecute, usePermissions, useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import ConfirmUpdate from "@/ui/config/confirm";
import UnsavedChanges from "@/ui/config/unsaved-changes";
import { Section, SectionProps } from "mogh_ui";
import { Button, Group } from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { notifications } from "@mantine/notifications";

export interface SwarmConfigEditProps extends SectionProps {
  swarm: string;
  config: string;
}

export default function SwarmConfigEditSection({
  swarm,
  config,
  ...sectionProps
}: SwarmConfigEditProps) {
  const [{ edit }, setEdit] = useLocalStorage<{ edit: string | undefined }>({
    key: `swarm-${swarm}-config-${config}-edit-v2`,
    defaultValue: { edit: undefined },
  });
  const {
    data: inspect,
    isPending,
    refetch,
    isError,
  } = useRead("InspectSwarmConfig", {
    swarm,
    config,
  });
  const { canExecute } = usePermissions({
    type: "Swarm",
    id: swarm,
  });
  const { mutateAsync: confirmEdit } = useExecute("RotateSwarmConfig", {
    onSuccess: (res) => {
      notifications.show({
        message: res.success ? "Config updated." : "Failed to update config.",
        color: res.success ? "green" : "red",
      });
      setEdit({ edit: undefined });
      refetch();
    },
  });

  const name = inspect?.Spec?.Name;
  const data = inspect?.Spec?.Data;
  const language = name ? languageFromPath(name) : undefined;

  return (
    <Section
      isPending={isPending}
      error={
        isError
          ? "Failed to inspect swarm config."
          : !config
            ? `No swarm config found with given name: ${config}`
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
              original={{ contents: data }}
              update={{ contents: edit }}
              onConfirm={async () =>
                name &&
                edit !== undefined &&
                (await confirmEdit({ swarm, config: name, data: edit }))
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
        value={edit ?? data ?? "Failed to retrieve config data"}
        language={language}
        onValueChange={(edit) => setEdit({ edit })}
        readOnly={!canExecute}
      />
    </Section>
  );
}
