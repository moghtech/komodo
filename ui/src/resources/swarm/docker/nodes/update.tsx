import { useExecute, useRead } from "@/lib/hooks";
import {
  Button,
  Group,
  Loader,
  Modal,
  Select,
  SimpleGrid,
  Stack,
  TagsInput,
  Text,
  useMatches,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useMemo, useState } from "react";
import { Types } from "komodo_client";
import { ICONS } from "@/theme/icons";
import LabelsGroup from "@/ui/labels-group";

export interface UpdateSwarmNodeProps {
  swarm: string;
  nodes: string[];
}

export default function UpdateSwarmNodes({
  swarm,
  nodes,
}: UpdateSwarmNodeProps) {
  const [opened, { open, close }] = useDisclosure();
  const { data: inspect, isPending } = useRead(
    "InspectSwarmNode",
    { swarm, node: nodes[0] },
    { enabled: opened && nodes.length === 1 },
  );

  const currentLabels = useMemo(() => {
    const labels = inspect?.Spec?.Labels ?? {};
    return Object.entries(labels).sort();
  }, [inspect]);

  const currentRole = inspect?.Spec?.Role ?? "";
  const currentAvailability = inspect?.Spec?.Availability ?? "";

  const [role, setRole] = useState<string | null>(null);
  const [availability, setAvailability] = useState<string | null>(null);
  const [labelsToAdd, setLabelsToAdd] = useState<string[]>([]);
  const [labelsToRemove, setLabelsToRemove] = useState<string[]>([]);

  const { mutate: update, isPending: updatePending } = useExecute(
    "UpdateSwarmNode",
    {
      onSuccess: () => {
        close();
        resetForm();
      },
    },
  );

  const resetForm = () => {
    setRole(null);
    setAvailability(null);
    setLabelsToAdd([]);
    setLabelsToRemove([]);
  };

  const onSubmit = () => {
    for (const node of nodes) {
      update({
        swarm,
        node,
        role: role ? (role as Types.NodeSpecRoleEnum) : undefined,
        availability: availability
          ? (availability as Types.NodeSpecAvailabilityEnum)
          : undefined,
        label_add: labelsToAdd.length > 0 ? labelsToAdd : undefined,
        label_rm: labelsToRemove.length > 0 ? labelsToRemove : undefined,
      });
    }
  };

  const tagSpan = useMatches({ base: 1, sm: 2 });

  return (
    <>
      <Modal
        opened={opened}
        onClose={() => {
          close();
          resetForm();
        }}
        title={`Update Node${nodes.length === 1 ? "" : "s"}: ${nodes.join(", ")}`}
        size="xl"
      >
        {nodes.length === 1 && isPending ? (
          <Group justify="center" p="xl">
            <Loader />
          </Group>
        ) : (
          <Stack>
            <SimpleGrid cols={{ base: 1, sm: 2 }}>
              <Select
                label="Role"
                placeholder={
                  (nodes.length === 1 &&
                    currentRole &&
                    `Current: ${currentRole}`) ||
                  "Select role"
                }
                data={[
                  { value: "worker", label: "Worker" },
                  { value: "manager", label: "Manager" },
                ]}
                value={role}
                onChange={setRole}
                clearable
              />

              <Select
                label="Availability"
                placeholder={
                  (nodes.length === 1 &&
                    currentAvailability &&
                    `Current: ${currentAvailability}`) ||
                  "Select availability"
                }
                data={[
                  { value: "active", label: "Active" },
                  { value: "pause", label: "Pause" },
                  { value: "drain", label: "Drain" },
                ]}
                value={availability}
                onChange={setAvailability}
                clearable
              />

              <TagsInput
                label="Labels to Add. Press enter to add multiple."
                placeholder="key=value"
                value={labelsToAdd}
                onChange={setLabelsToAdd}
              />

              <TagsInput
                label="Labels to Remove"
                placeholder="key"
                data={currentLabels.map(([key, _]) => key)}
                value={labelsToRemove}
                onChange={setLabelsToRemove}
              />

              {currentLabels.length > 0 && (
                <Stack gap="0.25rem" style={{ gridColumn: `span ${tagSpan}` }}>
                  <Text size="sm" fw={500}>
                    Current Labels
                  </Text>
                  <LabelsGroup labels={currentLabels} />
                </Stack>
              )}
            </SimpleGrid>
            <Group justify="flex-end" mt="md">
              <Button
                variant="default"
                onClick={() => {
                  close();
                  resetForm();
                }}
              >
                Cancel
              </Button>
              <Button onClick={onSubmit} loading={updatePending}>
                Update
              </Button>
            </Group>
          </Stack>
        )}
      </Modal>

      <Button
        onClick={open}
        disabled={!nodes.length}
        rightSection={<ICONS.Edit size="1rem" />}
        justify="space-between"
        w={{ base: "100%", xs: 190 }}
      >
        Update Node{nodes.length === 1 ? "" : "s"}
      </Button>
    </>
  );
}
