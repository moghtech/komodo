import { ConfirmButton, CopyButton } from "@components/util";
import { useInvalidate, useRead, useSetTitle, useWrite } from "@lib/hooks";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@ui/dialog";
import { Button } from "@ui/button";
import { useToast } from "@ui/use-toast";
import { Trash, PlusCircle, Loader2, Check, KeyRound } from "lucide-react";
import { useMemo, useState } from "react";
import { Input } from "@ui/input";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@ui/dropdown-menu";
import { Section } from "@components/layouts";
import { DataTable, SortableHeader } from "@ui/data-table";
import { fmt_date_with_minutes } from "@lib/formatting";
import { Switch } from "@ui/switch";
import { ResourceSelector, TagSelector } from "@components/resources/common";
import { Types } from "komodo_client";
import { ColumnDef } from "@tanstack/react-table";
import { Badge } from "@ui/badge";

export const Onboarding = () => {
  useSetTitle("Onboarding");
  const { toast } = useToast();
  const { data } = useRead("ListOnboardingKeys", {});
  const keys = data ?? [];
  const invalidate = useInvalidate();
  const { mutate } = useWrite("UpdateOnboardingKey", {
    onSuccess: () => {
      invalidate(["ListOnboardingKeys"]);
      toast({ title: "Updated onboarding key" });
    },
  });
  const columns: (
    | ColumnDef<Types.OnboardingKey, unknown>
    | false
    | undefined
  )[] = useMemo(
    () => [
      {
        size: 150,
        accessorKey: "name",
        header: ({ column }) => <SortableHeader column={column} title="Name" />,
        cell: ({ row }) => (
          <Input
            defaultValue={row.original.name}
            onBlur={(e) =>
              e.target.value != row.original.name &&
              mutate({
                public_key: row.original.public_key,
                name: e.target.value,
              })
            }
            onKeyDown={(e) => e.key === "Enter" && e.currentTarget.blur()}
          />
        ),
      },
      {
        size: 150,
        accessorKey: "copy_server",
        header: "Template",
        cell: ({ row }) => (
          <ResourceSelector
            type="Server"
            selected={row.original.copy_server}
            templates={Types.TemplatesQueryBehavior.Include}
            onSelect={(copy_server) =>
              mutate({ public_key: row.original.public_key, copy_server })
            }
          />
        ),
      },
      {
        size: 200,
        accessorKey: "tags",
        header: "Tags",
        cell: ({ row }) => (
          <TagSelector
            tags={row.original.tags!}
            set={(tags) =>
              mutate({ public_key: row.original.public_key, tags })
            }
            disabled={false}
            icon={<PlusCircle className="w-3 h-3" />}
            small
          />
        ),
      },
      {
        size: 100,
        accessorKey: "fix_existing_servers",
        header: ({ column }) => (
          <SortableHeader
            column={column}
            title="Priviledged"
            description="Allow the onboarding key to update an existing Server's public key and configuration to enable the connection."
          />
        ),
        cell: ({ row }) => (
          <Switch
            checked={row.original.fix_existing_servers}
            onCheckedChange={(fix_existing_servers) =>
              mutate({
                public_key: row.original.public_key,
                fix_existing_servers,
              })
            }
          />
        ),
      },
      {
        size: 100,
        accessorKey: "create_builder",
        header: ({ column }) => (
          <SortableHeader column={column} title="Create Builder" />
        ),
        cell: ({ row }) => (
          <Switch
            checked={row.original.create_builder}
            onCheckedChange={(create_builder) =>
              mutate({ public_key: row.original.public_key, create_builder })
            }
          />
        ),
      },
      {
        size: 100,
        accessorKey: "enabled",
        header: ({ column }) => (
          <SortableHeader column={column} title="Enabled" />
        ),
        cell: ({
          row: {
            original: { public_key, expires, enabled },
          },
        }) => (
          <Switch
            checked={expires && expires <= Date.now() ? false : enabled}
            onCheckedChange={(enabled) => mutate({ public_key, enabled })}
            disabled={!!expires && expires <= Date.now()}
          />
        ),
      },
      {
        size: 150,
        accessorKey: "expires",
        header: ({ column }) => (
          <SortableHeader column={column} title="Expires" />
        ),
        cell: ({
          row: {
            original: { expires },
          },
        }) => (
          <Badge
            variant={
              expires && expires <= Date.now() ? "destructive" : "secondary"
            }
          >
            {expires ? fmt_date_with_minutes(new Date(expires)) : "Never"}
          </Badge>
        ),
      },
      {
        size: 100,
        accessorKey: "public_key",
        header: "Delete",
        cell: ({ row }) => <DeleteKey public_key={row.original.public_key} />,
      },
    ],
    [mutate]
  );
  return (
    <Section
      title="Server Onboarding Keys"
      icon={<KeyRound className="w-5 h-5" />}
      actions={<CreateKey />}
      className="flex flex-col gap-6"
    >
      <DataTable
        tableKey="server-onboarding-keys-v1"
        data={keys}
        columns={columns}
      />
    </Section>
  );
};

const ONE_DAY_MS = 1000 * 60 * 60 * 24;

type ExpiresOptions = "1 day" | "7 days" | "30 days" | "never";

const CreateKey = () => {
  const { toast } = useToast();
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [privateKey, setPrivateKey] = useState("");
  const [expires, setExpires] = useState<ExpiresOptions>("1 day");
  const [submitted, setSubmitted] = useState<{ private_key: string }>();
  const invalidate = useInvalidate();
  const { mutate, isPending } = useWrite("CreateOnboardingKey", {
    onSuccess: ({ private_key }) => {
      toast({ title: "Onboarding Key Created" });
      invalidate(["ListOnboardingKeys"]);
      setSubmitted({ private_key });
    },
  });
  const now = Date.now();
  const expiresOptions: Record<ExpiresOptions, number> = {
    "1 day": now + ONE_DAY_MS,
    "7 days": now + ONE_DAY_MS * 7,
    "30 days": now + ONE_DAY_MS * 90,
    never: 0,
  };
  const submit = () =>
    mutate({
      name,
      expires: expiresOptions[expires],
      private_key: privateKey || undefined,
    });
  const onOpenChange = (open: boolean) => {
    setOpen(open);
    if (!open) {
      setName("");
      setExpires("never");
      setSubmitted(undefined);
    }
  };
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogTrigger asChild>
        <Button variant="secondary" className="items-center gap-2">
          New Onboarding Key <PlusCircle className="w-4 h-4" />
        </Button>
      </DialogTrigger>
      <DialogContent>
        {submitted ? (
          <>
            <DialogHeader>
              <DialogTitle>Onboarding Key Created</DialogTitle>
              <DialogDescription>
                Use as the PERIPHERY_ONBOARDING_KEY
              </DialogDescription>
            </DialogHeader>
            <div className="py-8 flex flex-col gap-4">
              <div className="flex items-center justify-between">
                Key
                <Input
                  className="w-72"
                  value={submitted.private_key}
                  disabled
                />
                <CopyButton content={submitted.private_key} />
              </div>
            </div>
            <DialogFooter className="flex justify-end">
              <Button
                variant="secondary"
                className="gap-4"
                onClick={() => onOpenChange(false)}
              >
                Confirm <Check className="w-4" />
              </Button>
            </DialogFooter>
          </>
        ) : (
          <>
            <DialogHeader>
              <DialogTitle>Create Onboarding Key</DialogTitle>
            </DialogHeader>
            <div className="py-8 flex flex-col gap-4">
              <div className="flex items-center justify-between">
                Name
                <Input
                  className="w-72"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="Optional"
                />
              </div>
              <div className="flex items-center justify-between">
                Pre-Existing Key
                <Input
                  className="w-72"
                  value={privateKey}
                  onChange={(e) => setPrivateKey(e.target.value)}
                  type="password"
                  placeholder="Optional"
                />
              </div>
              <div className="flex items-center justify-between">
                Expiry
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button
                      className="w-36 justify-between px-3"
                      variant="outline"
                    >
                      {expires}
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent className="w-36" side="bottom">
                    <DropdownMenuGroup>
                      {Object.keys(expiresOptions)
                        .filter((option) => option !== expires)
                        .map((option) => (
                          <DropdownMenuItem
                            key={option}
                            onClick={() => setExpires(option as any)}
                          >
                            {option}
                          </DropdownMenuItem>
                        ))}
                    </DropdownMenuGroup>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            </div>
            <DialogFooter className="flex justify-end">
              <Button
                variant="secondary"
                className="gap-4"
                onClick={submit}
                disabled={isPending}
              >
                Submit
                {isPending ? (
                  <Loader2 className="w-4 animate-spin" />
                ) : (
                  <Check className="w-4" />
                )}
              </Button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
};

const DeleteKey = ({ public_key }: { public_key: string }) => {
  const invalidate = useInvalidate();
  const { toast } = useToast();
  const { mutate, isPending } = useWrite("DeleteOnboardingKey", {
    onSuccess: () => {
      invalidate(["ListOnboardingKeys"]);
      toast({ title: "Onboarding Key Deleted" });
    },
  });
  return (
    <ConfirmButton
      title="Delete"
      variant="destructive"
      icon={<Trash className="w-4 h-4" />}
      onClick={(e) => {
        e.stopPropagation();
        mutate({ public_key });
      }}
      loading={isPending}
    />
  );
};
