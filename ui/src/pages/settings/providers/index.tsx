import {
  useInvalidate,
  useRead,
  useSetTitle,
  useUser,
  useWrite,
} from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { Section } from "mogh_ui";
import { Button, Group, Stack, Switch, Text } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { useState } from "react";
import NewProviderAccount from "./new";
import { DataTable, SortableHeader } from "mogh_ui";
import DeleteProviderAccount from "./delete";
import ProvidersFromConfig from "./from-config";
import { CopyButton } from "mogh_ui";
import { Types } from "komodo_client";
import { SharedTextUpdate, useSharedTextUpdateData } from "mogh_ui";
import { SearchInput } from "mogh_ui";

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

  const [updateMenuData, setUpdateMenuData] = useSharedTextUpdateData();

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
    <>
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
          <SearchInput value={search} onSearch={setSearch} />
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
                  <Group gap="sm" wrap="nowrap">
                    <Button
                      className="overflow-ellipsis"
                      onClick={() => {
                        setUpdateMenuData({
                          title: "Set Domain",
                          value: row.original.domain ?? "",
                          placeholder: "Input domain, eg. git.komo.do",
                          titleRight:
                            type === "GitProvider" ? (
                              <Group gap="xs">
                                <Text c="dimmed">Use HTTPS:</Text>
                                <Switch
                                  title="Use HTTPS"
                                  checked={
                                    (row.original as Types.GitProviderAccount)
                                      .https
                                  }
                                  onChange={(e) =>
                                    updateAccount({
                                      id: row.original._id?.$oid!,
                                      account: { https: e.target.checked },
                                    })
                                  }
                                />
                              </Group>
                            ) : undefined,
                          onUpdate: (domain) => {
                            if (row.original.domain === domain) {
                              return;
                            }
                            updateAccount({
                              id: row.original._id?.$oid!,
                              account: { domain },
                            });
                          },
                        });
                      }}
                      w={{ base: 200, lg: 300 }}
                      justify="start"
                    >
                      {row.original.domain || (
                        <Text c="dimmed">Set domain</Text>
                      )}
                    </Button>
                    <CopyButton content={row.original.domain ?? ""} />
                  </Group>
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
                  <Group gap="sm" wrap="nowrap">
                    <Button
                      className="overflow-ellipsis"
                      onClick={() => {
                        setUpdateMenuData({
                          title: "Set Username",
                          value: row.original.username ?? "",
                          placeholder: "Input account username",
                          onUpdate: (username) => {
                            if (row.original.username === username) {
                              return;
                            }
                            updateAccount({
                              id: row.original._id?.$oid!,
                              account: { username },
                            });
                          },
                        });
                      }}
                      w={{ base: 200, lg: 300 }}
                      justify="start"
                    >
                      {row.original.username || (
                        <Text c="dimmed">Set username</Text>
                      )}
                    </Button>
                    <CopyButton content={row.original.username ?? ""} />
                  </Group>
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
                  <Group gap="sm" wrap="nowrap">
                    <Button
                      className="overflow-ellipsis"
                      onClick={(e) => {
                        e.stopPropagation();
                        setUpdateMenuData({
                          title: "Set Token",
                          value: row.original.token ?? "",
                          placeholder: "Input account token",
                          onUpdate: (token) => {
                            if (row.original.token === token) {
                              return;
                            }
                            updateAccount({
                              id: row.original._id?.$oid!,
                              account: { token },
                            });
                          },
                        });
                      }}
                      w={{ base: 200, lg: 300 }}
                      justify="start"
                    >
                      {"*".repeat(row.original.token?.length || 0) || (
                        <Text c="dimmed">Set token</Text>
                      )}
                    </Button>
                    <CopyButton content={row.original.token ?? ""} />
                  </Group>
                );
              },
            },
            {
              header: "Delete",
              maxSize: 200,
              cell: ({ row }) => (
                <DeleteProviderAccount
                  type={type}
                  id={row.original._id?.$oid!}
                />
              ),
            },
          ]}
        />

        <ProvidersFromConfig type={type} />
      </Section>
      <SharedTextUpdate
        data={updateMenuData}
        setData={setUpdateMenuData}
        disabled={disabled}
      />
    </>
  );
}
