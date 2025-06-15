import {
  ConfirmButton,
  CopyButton,
  TextUpdateMenuMonaco,
} from "@components/util";
import {
  useInvalidate,
  useRead,
  useSetTitle,
  useUser,
  useWrite,
} from "@lib/hooks";
import { Types } from "komodo_client";
import { Button } from "@ui/button";
import { Card } from "@ui/card";
import { DataTable, SortableHeader } from "@ui/data-table";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@ui/dialog";
import { Input } from "@ui/input";
import { Switch } from "@ui/switch";
import { useToast } from "@ui/use-toast";
import {
  Check,
  GitBranch,
  HardDrive,
  Loader2,
  PlusCircle,
  Search,
  Trash,
} from "lucide-react";
import { ChangeEvent, ReactNode, useState } from "react";
import { Section } from "@components/layouts";
import { Badge } from "@ui/badge";

export const ProvidersPage = () => {
  return (
    <div className="flex flex-col gap-6">
      <Providers type="GitProvider" />
      <Providers type="DockerRegistry" />
    </div>
  );
};

const Providers = ({ type }: { type: "GitProvider" | "DockerRegistry" }) => {
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
            domain.includes(search) || (username && username.includes(search))
        );
      } else return true;
    }) ?? [];
  const { toast } = useToast();
  const inv = useInvalidate();
  const { mutate: updateAccount } = useWrite(`Update${type}Account`, {
    onSuccess: () => {
      inv([`List${type}Accounts`], [`Get${type}Account`]);
      toast({ title: "Updated account" });
    },
  });
  return (
    <Section
      title={type === "DockerRegistry" ? "Registry Accounts" : "Git Accounts"}
      icon={
        type === "DockerRegistry" ? (
          <HardDrive className="w-4 h-4" />
        ) : (
          <GitBranch className="w-4 h-4" />
        )
      }
    >
      {/* Create / Search */}
      <div className="flex items-center justify-between">
        <CreateAccount type={type} />
        <div className="relative">
          <Search className="w-4 absolute top-[50%] left-3 -translate-y-[50%] text-muted-foreground" />
          <Input
            placeholder="search..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-8 w-[200px] lg:w-[300px]"
          />
        </div>
      </div>

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
                <div className="flex items-center gap-2">
                  <Card
                    className="px-3 py-2 hover:bg-accent/50 transition-colors cursor-pointer w-full"
                    onClick={() => {
                      setUpdateMenuData({
                        title: "Set Domain",
                        value: row.original.domain ?? "",
                        placeholder: "Input domain, eg. git.komo.do",
                        titleRight:
                          type === "GitProvider" ? (
                            <UpdateHttps id={row.original._id?.$oid!} />
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
                  >
                    <div className="text-sm text-nowrap overflow-hidden overflow-ellipsis text-muted-foreground w-[100px] xl:w-[150px] 2xl:w-[200px]">
                      {row.original.domain || "Set domain"}
                    </div>
                  </Card>
                  <CopyButton content={row.original.domain} />
                </div>
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
                <div className="flex items-center gap-2">
                  <Card
                    className="px-3 py-2 hover:bg-accent/50 transition-colors cursor-pointer w-full"
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
                  >
                    <div className="text-sm text-nowrap overflow-hidden overflow-ellipsis text-muted-foreground w-[100px] xl:w-[150px] 2xl:w-[200px]">
                      {row.original.username || "Set username"}
                    </div>
                  </Card>
                  <CopyButton content={row.original.username} />
                </div>
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
                <div className="flex items-center gap-2">
                  <Card
                    className="px-3 py-2 hover:bg-accent/50 transition-colors cursor-pointer w-full"
                    onClick={() => {
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
                  >
                    <div className="text-sm text-nowrap overflow-hidden overflow-ellipsis text-muted-foreground w-[100px] xl:w-[150px] 2xl:w-[200px]">
                      {"*".repeat(row.original.token?.length || 0) ||
                        "Set token"}
                    </div>
                  </Card>
                  <CopyButton content={row.original.token} />
                </div>
              );
            },
          },
          {
            header: "Delete",
            maxSize: 200,
            cell: ({ row }) => (
              <DeleteAccount type={type} id={row.original._id?.$oid!} />
            ),
          },
        ]}
      />
      <ProvidersFromConfig type={type} />
      {updateMenuData && (
        <TextUpdateMenuMonaco
          title={updateMenuData.title}
          titleRight={updateMenuData.titleRight}
          placeholder={updateMenuData.placeholder}
          value={updateMenuData.value}
          onUpdate={updateMenuData.onUpdate}
          triggerClassName="w-full"
          disabled={disabled}
          open={!!updateMenuData}
          setOpen={(open) => {
            if (!open) {
              setUpdateMenuData(false);
            }
          }}
          triggerHidden
        />
      )}
    </Section>
  );
};

const ProvidersFromConfig = ({
  type,
}: {
  type: "GitProvider" | "DockerRegistry";
}) => {
  const accounts = useRead(
    type === "GitProvider"
      ? "ListGitProvidersFromConfig"
      : "ListDockerRegistriesFromConfig",
    {}
  )
    .data?.map((provider) =>
      provider.accounts.map((account) => [provider.domain, account.username])
    )
    .flat(1);
  if (!accounts)
    return (
      <div className="w-full flex justify-center">
        <Loader2 className="w-4 h-4 animate-spin" />
      </div>
    );
  if (accounts.length === 0) return;
  return (
    <div className="flex flex-col gap-2">
      <div className="text-muted-foreground">From config file:</div>
      <div className="flex gap-3 flex-wrap">
        {accounts.map(([domain, username]) => (
          <Badge variant="secondary">
            {domain} - {username}
          </Badge>
        ))}
      </div>
    </div>
  );
};

const UpdateHttps = ({ id }: { id: string }) => {
  const account = useRead("ListGitProviderAccounts", {}).data?.find(
    (account) => account._id?.$oid === id
  ) as Types.GitProviderAccount;
  const { toast } = useToast();
  const inv = useInvalidate();
  const { mutate: updateAccount } = useWrite("UpdateGitProviderAccount", {
    onSuccess: () => {
      inv(["ListGitProviderAccounts"], ["GetGitProviderAccount", { id }]);
      toast({ title: "Updated account" });
    },
  });
  return (
    <div className="flex items-center gap-2">
      <div>Https:</div>
      <Switch
        checked={account.https}
        onCheckedChange={(https) =>
          updateAccount({
            id,
            account: { https },
          })
        }
      />
    </div>
  );
};

const CreateAccount = ({
  type,
}: {
  type: "GitProvider" | "DockerRegistry";
}) => {
  const { toast } = useToast();
  const [open, setOpen] = useState(false);
  const [domain, setDomain] = useState("");
  const [https, setHttps] = useState(true);
  const [username, setUsername] = useState("");
  const [token, setToken] = useState("");
  const invalidate = useInvalidate();
  const { mutate: create, isPending } = useWrite(`Create${type}Account`, {
    onSuccess: () => {
      invalidate([`List${type}Accounts`]);
      toast({ title: "Account created" });
      setOpen(false);
    },
  });
  const submit = () => create({ account: { domain, https, username, token } });
  const form: Array<
    | undefined
    | [string, string, (e: ChangeEvent<HTMLInputElement>) => void, false]
    | [string, boolean, (checked: boolean) => void, true]
  > = [
    [
      "Domain",
      domain,
      (e: ChangeEvent<HTMLInputElement>) => setDomain(e.target.value),
      false,
    ],
    type === "GitProvider"
      ? ["Use https", https, (https: boolean) => setHttps(https), true]
      : undefined,
    [
      "Username",
      username,
      (e: ChangeEvent<HTMLInputElement>) => setUsername(e.target.value),
      false,
    ],
    [
      "Token",
      token,
      (e: ChangeEvent<HTMLInputElement>) => setToken(e.target.value),
      false,
    ],
  ];
  const account_type =
    type === "DockerRegistry" ? "Registry Account" : "Git Account";
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant="secondary" className="items-center gap-2">
          New Account <PlusCircle className="w-4 h-4" />
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create {account_type}</DialogTitle>
        </DialogHeader>
        <div className="py-8 flex flex-col gap-4">
          {form.map((item) => {
            if (!item) return;
            const [title, value, onChange, bool] = item;
            if (bool) {
              return (
                <div key={title} className="flex items-center justify-between">
                  {title}
                  <Switch
                    checked={value}
                    onCheckedChange={(checked) => onChange(checked)}
                  />
                </div>
              );
            }
            return (
              <div key={title} className="flex items-center justify-between">
                {title}
                <Input
                  placeholder={`Input ${title.toLowerCase()}`}
                  className="w-72"
                  value={value}
                  onChange={onChange}
                />
              </div>
            );
          })}
        </div>
        <DialogFooter className="flex justify-end">
          <Button className="gap-4" onClick={submit} disabled={isPending}>
            Create
            {isPending ? (
              <Loader2 className="w-4 animate-spin" />
            ) : (
              <Check className="w-4" />
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

const DeleteAccount = ({
  type,
  id,
}: {
  type: "GitProvider" | "DockerRegistry";
  id: string;
}) => {
  const invalidate = useInvalidate();
  const { toast } = useToast();
  const { mutate, isPending } = useWrite(`Delete${type}Account`, {
    onSuccess: () => {
      invalidate([`List${type}Accounts`], [`Get${type}Account`]);
      toast({ title: "Account deleted" });
    },
  });
  return (
    <ConfirmButton
      title="Delete"
      icon={<Trash className="w-4 h-4" />}
      onClick={() => mutate({ id })}
      loading={isPending}
    />
  );
};
