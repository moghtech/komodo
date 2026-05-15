import { languageFromPath, MonacoEditor } from "mogh_ui";
import { useExecute, useInvalidate, usePermissions } from "@/lib/hooks";
import { CreateModal } from "mogh_ui";
import { Stack, Text, TextInput } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { useMemo, useState } from "react";

export default function NewSwarmConfig({ id }: { id: string }) {
  const invalidate = useInvalidate();
  const { canExecute } = usePermissions({ type: "Swarm", id });
  const [name, setName] = useState("");
  const [data, setData] = useState("");
  const { mutateAsync: create, isPending } = useExecute("CreateSwarmConfig", {
    onSuccess: () => {
      invalidate(["ListSwarmConfigs"]);
      notifications.show({ message: "Swarm config created." });
      setName("");
      setData("");
    },
  });
  const language = useMemo(() => languageFromPath(name), [name]);
  return (
    <CreateModal
      modalSize="xl"
      entityType="Swarm Config"
      onConfirm={() => create({ swarm: id, name, data }).then(() => true)}
      disabled={!canExecute}
      loading={isPending}
      configureLabel="a unique name and content"
      configSection={() => (
        <Stack>
          <TextInput
            label="Name"
            autoFocus
            placeholder="config.yaml"
            value={name}
            onChange={(e) => setName(e.target.value)}
            error={!name.trim() && "Enter name"}
            maw={{ base: "100%", sm: 300 }}
          />
          <Stack gap="0.1rem">
            <Text size="sm">Content</Text>
            <MonacoEditor
              value={data}
              onValueChange={setData}
              language={language}
              maxHeightProportion={0.4}
            />
          </Stack>
        </Stack>
      )}
    />
  );
}
