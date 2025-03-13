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
import { Badge } from "@ui/badge";
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
import { Check, Loader2, PlusCircle, Search, Trash } from "lucide-react";
import { useState } from "react";

export const Variables = () => {
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
  const searchSplit = search?.toLowerCase().split(" ") || [];
  const filtered =
    variables?.filter((variable) => {
      if (searchSplit.length > 0) {
        const name = variable.name.toLowerCase();
        return searchSplit.every((search) => name.includes(search));
      } else return true;
    }) ?? [];
  const { toast } = useToast();
  const inv = useInvalidate();
  const { mutate: updateValue } = useWrite("UpdateVariableValue", {
    onSuccess: () => {
      inv(["ListVariables"], ["GetVariable"]);
      toast({ title: "Updated variable value" });
    },
  });
  const { mutate: updateDescription } = useWrite("UpdateVariableDescription", {
    onSuccess: () => {
      inv(["ListVariables"], ["GetVariable"]);
      toast({ title: "Updated variable description" });
    },
  });
  const { mutate: updateIsSecret } = useWrite("UpdateVariableIsSecret", {
    onSuccess: () => {
      inv(["ListVariables"], ["GetVariable"]);
      toast({ title: "Updated variable 'is secret'" });
    },
  });
  return (
    <div className="flex flex-col gap-4">
      <div className="flex justify-between gap-4">
        <CreateVariable />
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

      {updateMenuData && (
        <TextUpdateMenuMonaco
          title={updateMenuData.title}
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

      {/** VARIABLES */}
      <div className="max-w-full overflow-auto">
        {/* <div className="w-full min-w-[1200px]"> */}
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
            {
              accessorKey: "value",
              size: 300,
              header: ({ column }) => (
                <SortableHeader column={column} title="Value" />
              ),
              cell: ({ row }) => {
                const valueDisplay = row.original.is_secret
                  ? "*".repeat(row.original.value?.length || 0)
                  : row.original.value;
                return (
                  <div className="flex items-center gap-2">
                    <Card
                      className="w-full max-w-[200px] xl:max-w-full px-3 py-2 hover:bg-accent/50 transition-colors cursor-pointer text-sm text-nowrap overflow-hidden overflow-ellipsis text-muted-foreground"
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
                    >
                      {valueDisplay || "Set value"}
                    </Card>
                    <CopyButton content={row.original.value} />
                  </div>
                );
              },
            },
            {
              accessorKey: "description",
              size: 200,
              header: "Description",
              cell: ({ row }) => {
                return (
                  <Card
                    className="px-3 py-2 hover:bg-accent/50 transition-colors cursor-pointer w-full"
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
                  >
                    <div className="text-sm text-nowrap overflow-hidden overflow-ellipsis w-full text-muted-foreground">
                      {row.original.description || "Set description"}
                    </div>
                  </Card>
                );
              },
            },
            {
              header: "Secret",
              size: 100,
              cell: ({ row }) => (
                <Switch
                  checked={row.original.is_secret}
                  onCheckedChange={(is_secret) =>
                    updateIsSecret({ name: row.original.name, is_secret })
                  }
                  disabled={disabled}
                />
              ),
            },
            {
              header: "Delete",
              size: 200,
              cell: ({ row }) => <DeleteVariable name={row.original.name} />,
            },
          ]}
        />
        {/* </div> */}
      </div>

      {/** SECRETS */}
      {secrets.length ? (
        <div className="flex items-center gap-2 flex-wrap text-muted-foreground">
          <div>Core Secrets:</div>
          {secrets.map((secret) => (
            <Badge variant="secondary">{secret}</Badge>
          ))}
        </div>
      ) : undefined}
    </div>
  );
};

const CreateVariable = () => {
  const { toast } = useToast();
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const invalidate = useInvalidate();
  const { mutate, isPending } = useWrite("CreateVariable", {
    onSuccess: () => {
      invalidate(["ListVariables"], ["GetVariable"]);
      toast({ title: "Variable Created" });
      setOpen(false);
    },
  });
  const user = useUser().data;
  const disabled = !user?.admin;
  const submit = () => mutate({ name });
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button
          variant="secondary"
          className="items-center gap-2"
          disabled={disabled}
        >
          New Variable <PlusCircle className="w-4 h-4" />
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create Variable</DialogTitle>
        </DialogHeader>
        <div className="py-8 flex flex-col gap-4">
          <div className="flex items-center justify-between">
            Name
            <Input
              className="w-72"
              value={name}
              onChange={(e) =>
                setName(e.target.value.toUpperCase().replaceAll(" ", "_"))
              }
              placeholder="Input variable name"
            />
          </div>
        </div>
        <DialogFooter className="flex justify-end">
          <Button className="gap-4" onClick={submit} disabled={isPending}>
            Submit
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

const DeleteVariable = ({ name }: { name: string }) => {
  const invalidate = useInvalidate();
  const { toast } = useToast();
  const { mutate, isPending } = useWrite("DeleteVariable", {
    onSuccess: () => {
      invalidate(["ListVariables"], ["GetVariable"]);
      toast({ title: "Variable deleted" });
    },
  });
  return (
    <ConfirmButton
      title="Delete"
      icon={<Trash className="w-4 h-4" />}
      onClick={() => mutate({ name })}
      loading={isPending}
    />
  );
};
