import NewVariable from "@/pages/settings/variables/new";
import {
  useInvalidate,
  useRead,
  useSetTitle,
  useUser,
  useWrite,
} from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import { ICONS } from "@/theme/icons";
import CopyButton from "@/ui/copy-button";
import { DataTable, SortableHeader } from "@/ui/data-table";
import {
  Badge,
  Button,
  Group,
  Stack,
  Switch,
  Text,
  TextInput,
} from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { useState } from "react";
import DeleteVariable from "./delete";
import SharedTextUpdate, {
  useSharedTextUpdateData,
} from "@/ui/shared-text-update";

export default function SettingsVariables() {
  const user = useUser().data;
  const disabled = !user?.admin;
  useSetTitle("Variables");
  const [updateMenuData, setUpdateMenuData] = useSharedTextUpdateData();

  const [search, setSearch] = useState("");

  const variables = useRead("ListVariables", {}).data ?? [];
  const secrets = useRead("ListSecrets", {}).data ?? [];

  const filtered = filterBySplit(variables, search, (item) => item.name);

  const inv = useInvalidate();

  const { mutate: updateValue } = useWrite("UpdateVariableValue", {
    onSuccess: () => {
      inv(["ListVariables"], ["GetVariable"]);
      notifications.show({ message: "Updated variable value" });
    },
  });

  const { mutate: updateDescription } = useWrite("UpdateVariableDescription", {
    onSuccess: () => {
      inv(["ListVariables"], ["GetVariable"]);
      notifications.show({ message: "Updated variable description" });
    },
  });

  const { mutate: updateIsSecret } = useWrite("UpdateVariableIsSecret", {
    onSuccess: () => {
      inv(["ListVariables"], ["GetVariable"]);
      notifications.show({ message: "Updated variable 'is secret'" });
    },
  });

  return (
    <>
      <Stack>
        <Group justify="space-between">
          <NewVariable />
          <TextInput
            placeholder="search..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            w={{ base: 200, lg: 300 }}
            leftSection={<ICONS.Search size="1rem" />}
          />
        </Group>

        <DataTable
          tableKey="variables"
          data={filtered}
          columns={[
            {
              accessorKey: "name",
              size: 200,
              header: ({ column }) => (
                <SortableHeader column={column} title="Name" />
              ),
              cell: ({ row }) => {
                return (
                  <Group gap="sm" wrap="nowrap">
                    <Text
                      title={row.original.name}
                      w={{ base: 200, lg: 300 }}
                      p="xs"
                      bdrs="sm"
                      style={{ overflow: "hidden", whiteSpace: "nowrap" }}
                      className="text-ellipsis bordered-light"
                      size="sm"
                    >
                      {row.original.name}
                    </Text>
                    <CopyButton content={row.original.name} />
                  </Group>
                );
              },
            },
            {
              accessorKey: "value",
              size: 300,
              header: ({ column }) => (
                <SortableHeader column={column} title="Value" />
              ),
              cell: ({ row }) => {
                return (
                  <Group gap="sm" wrap="nowrap">
                    <Button
                      className="text-ellipsis"
                      onClick={() => {
                        setUpdateMenuData({
                          title: `${row.original.name} - Value`,
                          value: row.original.value ?? "",
                          placeholder: "Set value",
                          onUpdate: (value) => {
                            if (row.original.value === value) {
                              return;
                            }
                            updateValue({ name: row.original.name, value });
                          },
                        });
                      }}
                      w={{ base: 200, lg: 300 }}
                      justify="start"
                    >
                      {(row.original.is_secret
                        ? "*".repeat(row.original.value?.length ?? 0)
                        : row.original.value) || (
                        <Text c="dimmed">Set value</Text>
                      )}
                    </Button>
                    <CopyButton content={row.original.value ?? ""} />
                  </Group>
                );
              },
            },
            {
              accessorKey: "description",
              size: 200,
              header: ({ column }) => (
                <SortableHeader column={column} title="Description" />
              ),
              cell: ({ row }) => {
                return (
                  <Button
                    className="text-ellipsis"
                    onClick={() => {
                      setUpdateMenuData({
                        title: `${row.original.name} - Description`,
                        value: row.original.description ?? "",
                        placeholder: "Set description",
                        onUpdate: (description) => {
                          if (row.original.description === description) {
                            return;
                          }
                          updateDescription({
                            name: row.original.name,
                            description,
                          });
                        },
                      });
                    }}
                    w={{ base: 200, lg: 300 }}
                    justify="start"
                  >
                    {row.original.description || (
                      <Text c="dimmed">Set description</Text>
                    )}
                  </Button>
                );
              },
            },
            {
              header: "Secret",
              size: 100,
              cell: ({ row }) => (
                <Switch
                  checked={row.original.is_secret}
                  onChange={(e) =>
                    updateIsSecret({
                      name: row.original.name,
                      is_secret: e.target.checked,
                    })
                  }
                  disabled={disabled}
                />
              ),
            },
            {
              header: "Delete",
              size: 200,
              cell: ({ row }) => (
                <DeleteVariable name={row.original.name} disabled={disabled} />
              ),
            },
          ]}
        />

        {secrets?.length ? (
          <Group>
            <Text c="dimmed">Core Secrets:</Text>
            <Group gap="xs">
              {secrets.map((secret) => (
                <Badge key={secret} tt="none">
                  {secret}
                </Badge>
              ))}
            </Group>
          </Group>
        ) : undefined}
      </Stack>

      <SharedTextUpdate
        data={updateMenuData}
        setData={setUpdateMenuData}
        disabled={disabled}
      />
    </>
  );
}
