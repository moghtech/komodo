import { ConfirmButton } from "@components/util";
import { useManageAuth, useUserInvalidate } from "@lib/hooks";
import { Button } from "@ui/button";
import { Fingerprint, Trash } from "lucide-react";
import { MoghAuth, Types } from "komodo_client";
import { cn } from "@lib/utils";
import { useToast } from "@ui/use-toast";

export const EnrollPasskey = ({ user }: { user: Types.User }) => {
  const userInvalidate = useUserInvalidate();
  const { toast } = useToast();

  const { mutate: unenroll, isPending: unenroll_pending } = useManageAuth(
    "UnenrollPasskey",
    {
      onSuccess: () => {
        userInvalidate();
        toast({ title: "Unenrolled in passkey 2FA" });
      },
    },
  );

  const { mutate: confirm_enrollment } = useManageAuth(
    "ConfirmPasskeyEnrollment",
    {
      onSuccess: () => {
        userInvalidate();
        toast({ title: "Enrolled in passkey authentication" });
      },
    },
  );

  const { mutate: begin_enrollment } = useManageAuth("BeginPasskeyEnrollment", {
    onSuccess: (challenge) => {
      navigator.credentials
        .create(MoghAuth.Passkey.prepareCreationChallengeResponse(challenge))
        .then((credential) => confirm_enrollment({ credential }))
        .catch((e) => {
          console.error(e);
          toast({
            title: "Failed to create passkey",
            description: "See console for details",
            variant: "destructive",
          });
        });
    },
  });

  return (
    <>
      <Button
        variant="secondary"
        className={cn(
          "items-center gap-2",
          (user.totp?.confirmed_at || !!user.passkey?.created_at) && "hidden",
        )}
        onClick={() => begin_enrollment({})}
      >
        Enroll Passkey 2FA <Fingerprint className="w-4 h-4" />
      </Button>
      <ConfirmButton
        className={!user.passkey?.created_at ? "hidden" : "max-w-[220px]"}
        variant="destructive"
        title="Unenroll Passkey 2FA"
        icon={<Trash className="w-4 h-4" />}
        loading={unenroll_pending}
        onClick={() => unenroll({})}
      />
    </>
  );
};
