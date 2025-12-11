import { ConfirmButton, CopyButton } from "@components/util";
import { useManageUser, useUserInvalidate } from "@lib/hooks";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@ui/dialog";
import { Button } from "@ui/button";
import { Loader2, Check, Trash, RotateCcwKey } from "lucide-react";
import { useState } from "react";
import { Input } from "@ui/input";
import { Types } from "komodo_client";
import { cn } from "@lib/utils";
import { useToast } from "@ui/use-toast";

export const EnrollTotp = ({ user }: { user: Types.User }) => {
  const userInvalidate = useUserInvalidate();
  const { toast } = useToast();
  const [open, setOpen] = useState(false);
  const [submitted, setSubmitted] = useState<{ uri: string; png: string }>();
  const [confirm, setConfirm] = useState("");
  const [recovery, setRecovery] = useState<string[] | undefined>(undefined);
  const { mutate: begin_enrollment } = useManageUser("BeginTotpEnrollment", {
    onSuccess: ({ uri, png }) => setSubmitted({ uri, png }),
  });
  const { mutate: confirm_enrollment, isPending: confirm_pending } =
    useManageUser("ConfirmTotpEnrollment", {
      onSuccess: ({ recovery_codes }) => {
        setRecovery(recovery_codes);
        userInvalidate();
      },
    });
  const { mutate: unenroll, isPending: unenroll_pending } = useManageUser(
    "UnenrollTotp",
    {
      onSuccess: () => {
        userInvalidate();
        toast({ title: "Unenrolled in TOTP 2FA" });
      },
    }
  );
  const onOpenChange = (open: boolean) => {
    setOpen(open);
    if (open) {
      begin_enrollment({});
    } else {
      setSubmitted(undefined);
      setRecovery(undefined);
    }
  };
  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogTrigger asChild>
          <Button
            variant="secondary"
            className={cn(
              "items-center gap-2",
              (user.passkey?.created_at || !!user.totp?.confirmed_at) &&
                "hidden"
            )}
          >
            Enroll TOTP 2FA <RotateCcwKey className="w-4 h-4" />
          </Button>
        </DialogTrigger>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Enroll TOTP 2FA</DialogTitle>
          </DialogHeader>
          {recovery ? (
            <>
              <div className="py-8 flex flex-col gap-4">
                <h2>Save recovery keys</h2>
                {recovery.map((code) => (
                  <Input key={code} className="w-72" value={code} disabled />
                ))}
                <CopyButton content={recovery.join("\n")} />
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
          ) : submitted ? (
            <>
              <div className="py-8 flex flex-col gap-4">
                <h2>
                  Scan this QR code with your authenticator app, and enter the 6
                  digit code below.
                </h2>
                <div className="flex items-center justify-center">
                  <img
                    className="w-72"
                    src={"data:image/png;base64," + submitted.png}
                    alt="QR"
                  />
                </div>
                <div className="flex items-center justify-between">
                  URI
                  <Input className="w-72" value={submitted.uri} disabled />
                  <CopyButton content={submitted.uri} />
                </div>
                <div className="flex items-center justify-between">
                  Confirm Code
                  <Input
                    className="w-72"
                    value={confirm}
                    onChange={(e) => setConfirm(e.target.value)}
                    autoFocus
                  />
                </div>
              </div>
              <DialogFooter className="flex justify-end">
                <Button
                  variant="secondary"
                  className="gap-4"
                  onClick={() => confirm_enrollment({ code: confirm })}
                  disabled={confirm.length !== 6 || confirm_pending}
                >
                  Confirm{" "}
                  {confirm_pending ? (
                    <Loader2 className="w-4 animate-spin" />
                  ) : (
                    <Check className="w-4" />
                  )}
                </Button>
              </DialogFooter>
            </>
          ) : (
            <>
              <DialogHeader>
                <DialogTitle>Create Api Key</DialogTitle>
              </DialogHeader>
              <div className="py-8 flex justify-center gap-4">
                <Loader2 className="w-8 animate-spin py-2" />
              </div>
              <DialogFooter className="flex justify-end">
                <Button variant="secondary" className="gap-4" disabled>
                  Confirm
                  <Loader2 className="w-4 animate-spin" />
                </Button>
              </DialogFooter>
            </>
          )}
        </DialogContent>
      </Dialog>
      <ConfirmButton
        className={!user.totp?.confirmed_at ? "hidden" : undefined}
        variant="destructive"
        title="Unenroll TOTP 2FA"
        icon={<Trash className="w-4 h-4" />}
        loading={unenroll_pending}
        onClick={() => unenroll({})}
      />
    </>
  );
};
