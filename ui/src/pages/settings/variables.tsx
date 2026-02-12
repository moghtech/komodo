import ExportToml from "@/components/export-toml";
import NewVariable from "@/components/variables/new";
import {
  useInvalidate,
  useRead,
  useSetTitle,
  useUser,
  useWrite,
} from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import { ICONS } from "@/theme/icons";
import { DataTable, SortableHeader } from "@/ui/data-table";
import TextUpdateModal from "@/ui/text-update-modal";
import { Group, Stack, Switch, TextInput } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { useState } from "react";

export default function SettingsVariables() {
  const user = useUser().data;
  const disabled = !user?.admin;
  useSetTitle("Variables");

  const [updateMenuData, setUpdateMenuData] = useState<
    | false
    | {
        title: string;
        value: string;
        placeholder: string;
        onUpdate: (value: string) => void;
      }
  >(false);

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
          },
          // {
          //   accessorKey: "value",
          //   size: 300,
          //   header: ({ column }) => (
          //     <SortableHeader column={column} title="Value" />
          //   ),
          //   cell: ({ row }) => {
          //     const valueDisplay = row.original.is_secret
          //       ? "*".repeat(row.original.value?.length || 0)
          //       : row.original.value;
          //     return (
          //       <div className="flex items-center gap-2">
          //         <Card
          //           className="w-full max-w-[200px] xl:max-w-full px-3 py-2 hover:bg-accent/50 transition-colors cursor-pointer text-sm text-nowrap overflow-hidden overflow-ellipsis text-muted-foreground"
          //           onClick={() => {
          //             setUpdateMenuData({
          //               title: `${row.original.name} - Value`,
          //               value: row.original.value ?? "",
          //               placeholder: "Set value",
          //               onUpdate: (value) => {
          //                 if (row.original.value === value) {
          //                   return;
          //                 }
          //                 updateValue({ name: row.original.name, value });
          //               },
          //             });
          //           }}
          //         >
          //           {valueDisplay || "Set value"}
          //         </Card>
          //         <CopyButton content={row.original.value} />
          //       </div>
          //     );
          //   },
          // },
          // {
          //   accessorKey: "description",
          //   size: 200,
          //   header: "Description",
          //   cell: ({ row }) => {
          //     return (
          //       <Card
          //         className="px-3 py-2 hover:bg-accent/50 transition-colors cursor-pointer w-full"
          //         onClick={() => {
          //           setUpdateMenuData({
          //             title: `${row.original.name} - Description`,
          //             value: row.original.description ?? "",
          //             placeholder: "Set description",
          //             onUpdate: (description) => {
          //               if (row.original.description === description) {
          //                 return;
          //               }
          //               updateDescription({
          //                 name: row.original.name,
          //                 description,
          //               });
          //             },
          //           });
          //         }}
          //       >
          //         <div className="text-sm text-nowrap overflow-hidden overflow-ellipsis w-full text-muted-foreground">
          //           {row.original.description || "Set description"}
          //         </div>
          //       </Card>
          //     );
          //   },
          // },
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
          // {
          //   header: "Delete",
          //   size: 200,
          //   cell: ({ row }) => <DeleteVariable name={row.original.name} />,
          // },
        ]}
      />
    </Stack>
  );
}
