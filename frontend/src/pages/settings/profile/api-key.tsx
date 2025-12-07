import { ConfirmButton, CopyButton } from "@components/util";
import {
  useInvalidate,
  useManageUser,
} from "@lib/hooks";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@ui/dialog";
import { Button } from "@ui/button";
import { useToast } from "@ui/use-toast";
import {
  Trash,
  PlusCircle,
  Loader2,
  Check,
} from "lucide-react";
import { useState } from "react";
import { Input } from "@ui/input";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@ui/dropdown-menu";

const ONE_DAY_MS = 1000 * 60 * 60 * 24;

type ExpiresOptions = "90 days" | "180 days" | "1 year" | "never";

export const CreateKey = () => {
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [expires, setExpires] = useState<ExpiresOptions>("never");
  const [submitted, setSubmitted] = useState<{ key: string; secret: string }>();
  const invalidate = useInvalidate();
  const { mutate, isPending } = useManageUser("CreateApiKey", {
    onSuccess: ({ key, secret }) => {
      invalidate(["ListApiKeys"]);
      setSubmitted({ key, secret });
    },
  });
  const now = Date.now();
  const expiresOptions: Record<ExpiresOptions, number> = {
    "90 days": now + ONE_DAY_MS * 90,
    "180 days": now + ONE_DAY_MS * 180,
    "1 year": now + ONE_DAY_MS * 365,
    never: 0,
  };
  const submit = () => mutate({ name, expires: expiresOptions[expires] });
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
          New Api Key <PlusCircle className="w-4 h-4" />
        </Button>
      </DialogTrigger>
      <DialogContent>
        {submitted ? (
          <>
            <DialogHeader>
              <DialogTitle>Api Key Created</DialogTitle>
            </DialogHeader>
            <div className="py-8 flex flex-col gap-4">
              <div className="flex items-center justify-between">
                Key
                <Input className="w-72" value={submitted.key} disabled />
                <CopyButton content={submitted.key} />
              </div>
              <div className="flex items-center justify-between">
                Secret
                <Input className="w-72" value={submitted.secret} disabled />
                <CopyButton content={submitted.secret} />
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
              <DialogTitle>Create Api Key</DialogTitle>
            </DialogHeader>
            <div className="py-8 flex flex-col gap-4">
              <div className="flex items-center justify-between">
                Name
                <Input
                  className="w-72"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
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

export const DeleteKey = ({ api_key }: { api_key: string }) => {
  const invalidate = useInvalidate();
  const { toast } = useToast();
  const { mutate, isPending } = useManageUser("DeleteApiKey", {
    onSuccess: () => {
      invalidate(["ListApiKeys"]);
      toast({ title: "Api Key Deleted" });
    },
    onError: () => {
      toast({ title: "Failed to delete api key", variant: "destructive" });
    },
  });
  return (
    <ConfirmButton
      title="Delete"
      variant="destructive"
      icon={<Trash className="w-4 h-4" />}
      onClick={(e) => {
        e.stopPropagation();
        mutate({ key: api_key });
      }}
      loading={isPending}
    />
  );
};
