import { CopyButton } from "@components/util";
import { useManageUser } from "@lib/hooks";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@ui/dialog";
import { Button } from "@ui/button";
import { PlusCircle, Loader2, Check } from "lucide-react";
import { useState } from "react";
import { Input } from "@ui/input";

export const EnrollTotp = () => {
  const [open, setOpen] = useState(false);
  const [submitted, setSubmitted] = useState<{ uri: string; png: string }>();
  const [confirm, setConfirm] = useState("");
  const [recovery, setRecovery] = useState<string[] | undefined>(undefined);
  const { mutate: begin_enrollment } = useManageUser("BeginTotpEnrollment", {
    onSuccess: ({ uri, png }) => setSubmitted({ uri, png }),
  });
  const { mutate: confirm_enrollment, isPending: confirm_pending } =
    useManageUser("ConfirmTotpEnrollment", {
      onSuccess: ({ recovery_codes }) => setRecovery(recovery_codes),
    });
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
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogTrigger asChild>
        <Button variant="secondary" className="items-center gap-2">
          Enroll TOTP 2FA <PlusCircle className="w-4 h-4" />
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Enroll TOTP 2FA</DialogTitle>
        </DialogHeader>
        {recovery ? (
          <>
            <div className="py-8 flex flex-col gap-4">
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
  );
};
