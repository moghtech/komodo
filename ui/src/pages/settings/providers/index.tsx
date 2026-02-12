import {
  useInvalidate,
  useRead,
  useSetTitle,
  useUser,
  useWrite,
} from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import Section from "@/ui/section";
import { Group, Stack, TextInput } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { ReactNode, useState } from "react";
import NewProviderAccount from "./new";
import { DataTable, SortableHeader } from "@/ui/data-table";
import DeleteProviderAccount from "./delete";
import ProvidersFromConfig from "./from-config";

export default function SettingsProviders() {
  return (
    <Stack gap="xl">
      <Providers type="GitProvider" />
      <Providers type="DockerRegistry" />
    </Stack>
  );
}

function Providers({ type }: { type: "GitProvider" | "DockerRegistry" }) {
  const user = useUser().data;
  const disabled = !user?.admin;
  useSetTitle("Providers");
  const [updateMenuData, setUpdateMenuData] = useState<
    | false
    | {
        title: string;
        value: string;
        placeholder: string;
        onUpdate: (value: string) => void;
        titleRight?: ReactNode;
      }
  >(false);
  const [search, setSearch] = useState("");
  const accounts = useRead(`List${type}Accounts`, {}).data ?? [];
  const searchSplit = search?.toLowerCase().split(" ") || [];
  const filtered =
    accounts?.filter((account) => {
      if (searchSplit.length > 0) {
        const domain = account.domain?.toLowerCase();
        const username = account.username?.toLowerCase();
        return searchSplit.every(
          (search) =>
            domain.includes(search) || (username && username.includes(search)),
        );
      } else return true;
    }) ?? [];
  const inv = useInvalidate();
  const { mutate: updateAccount } = useWrite(`Update${type}Account`, {
    onSuccess: () => {
      inv([`List${type}Accounts`], [`Get${type}Account`]);
      notifications.show({ message: "Updated account" });
    },
  });

  return (
    <Section
      title={type === "DockerRegistry" ? "Registry Accounts" : "Git Accounts"}
      icon={
        type === "DockerRegistry" ? (
          <ICONS.Image size="1.3rem" />
        ) : (
          <ICONS.Repo size="1.3rem" />
        )
      }
    >
      <Group justify="space-between">
        <NewProviderAccount type={type} />
        <TextInput
          placeholder="search..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          w={{ base: 200, lg: 300 }}
          leftSection={<ICONS.Search size="1rem" />}
        />
      </Group>

      {/* ACCOUNTS */}
      <DataTable
        tableKey={type + "-accounts"}
        data={filtered}
        columns={[
          {
            accessorKey: "domain",
            size: 200,
            header: ({ column }) => (
              <SortableHeader column={column} title="Domain" />
            ),
            cell: ({ row }) => {
              return (
                <></>
                // <div className="flex items-center gap-2">
                //   <Card
                //     className="px-3 py-2 hover:bg-accent/50 transition-colors cursor-pointer w-full"
                //     onClick={() => {
                //       setUpdateMenuData({
                //         title: "Set Domain",
                //         value: row.original.domain ?? "",
                //         placeholder: "Input domain, eg. git.komo.do",
                //         titleRight:
                //           type === "GitProvider" ? (
                //             <UpdateHttps id={row.original._id?.$oid!} />
                //           ) : undefined,
                //         onUpdate: (domain) => {
                //           if (row.original.domain === domain) {
                //             return;
                //           }
                //           updateAccount({
                //             id: row.original._id?.$oid!,
                //             account: { domain },
                //           });
                //         },
                //       });
                //     }}
                //   >
                //     <div className="text-sm text-nowrap overflow-hidden overflow-ellipsis text-muted-foreground w-[100px] xl:w-[150px] 2xl:w-[200px]">
                //       {row.original.domain || "Set domain"}
                //     </div>
                //   </Card>
                //   <CopyButton content={row.original.domain} />
                // </div>
              );
            },
          },
          {
            accessorKey: "username",
            size: 200,
            header: ({ column }) => (
              <SortableHeader column={column} title="Username" />
            ),
            cell: ({ row }) => {
              return (
                <></>
                // <div className="flex items-center gap-2">
                //   <Card
                //     className="px-3 py-2 hover:bg-accent/50 transition-colors cursor-pointer w-full"
                //     onClick={() => {
                //       setUpdateMenuData({
                //         title: "Set Username",
                //         value: row.original.username ?? "",
                //         placeholder: "Input account username",
                //         onUpdate: (username) => {
                //           if (row.original.username === username) {
                //             return;
                //           }
                //           updateAccount({
                //             id: row.original._id?.$oid!,
                //             account: { username },
                //           });
                //         },
                //       });
                //     }}
                //   >
                //     <div className="text-sm text-nowrap overflow-hidden overflow-ellipsis text-muted-foreground w-[100px] xl:w-[150px] 2xl:w-[200px]">
                //       {row.original.username || "Set username"}
                //     </div>
                //   </Card>
                //   <CopyButton content={row.original.username} />
                // </div>
              );
            },
          },
          {
            accessorKey: "token",
            size: 200,
            header: ({ column }) => (
              <SortableHeader column={column} title="Token" />
            ),
            cell: ({ row }) => {
              return (
                <></>
                // <div className="flex items-center gap-2">
                //   <Card
                //     className="px-3 py-2 hover:bg-accent/50 transition-colors cursor-pointer w-full"
                //     onClick={() => {
                //       setUpdateMenuData({
                //         title: "Set Token",
                //         value: row.original.token ?? "",
                //         placeholder: "Input account token",
                //         onUpdate: (token) => {
                //           if (row.original.token === token) {
                //             return;
                //           }
                //           updateAccount({
                //             id: row.original._id?.$oid!,
                //             account: { token },
                //           });
                //         },
                //       });
                //     }}
                //   >
                //     <div className="text-sm text-nowrap overflow-hidden overflow-ellipsis text-muted-foreground w-[100px] xl:w-[150px] 2xl:w-[200px]">
                //       {"*".repeat(row.original.token?.length || 0) ||
                //         "Set token"}
                //     </div>
                //   </Card>
                //   <CopyButton content={row.original.token} />
                // </div>
              );
            },
          },
          {
            header: "Delete",
            maxSize: 200,
            cell: ({ row }) => (
              <DeleteProviderAccount type={type} id={row.original._id?.$oid!} />
            ),
          },
        ]}
      />

      <ProvidersFromConfig type={type} />
    </Section>
  );
}
