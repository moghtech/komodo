import { ConfirmButton } from "@components/util";
import { useManageUser, useUserInvalidate } from "@lib/hooks";
import { Button } from "@ui/button";
import { PlusCircle, Trash } from "lucide-react";
import { Types } from "komodo_client";
import { base64urlToArrayBuffer, cn } from "@lib/utils";
import { useToast } from "@ui/use-toast";

export const EnrollPasskey = ({ user }: { user: Types.User }) => {
  const userInvalidate = useUserInvalidate();
  const { toast } = useToast();

  const { mutate: unenroll, isPending: unenroll_pending } = useManageUser(
    "UnenrollPasskey",
    {
      onSuccess: () => {
        userInvalidate();
        toast({ title: "Unenrolled in passkey authentication" });
      },
    }
  );

  const { mutate: confirm_enrollment } = useManageUser(
    "ConfirmPasskeyEnrollment",
    {
      onSuccess: () => {
        userInvalidate();
        toast({ title: "Enrolled in passkey authentication" });
      },
    }
  );

  const { mutate: begin_enrollment } = useManageUser("BeginPasskeyEnrollment", {
    onSuccess: (challenge) => {
      const formatted_challenge = {
        ...challenge,
        publicKey: {
          ...challenge.publicKey,
          challenge: base64urlToArrayBuffer(challenge.publicKey.challenge),
          user: {
            ...challenge.publicKey.user,
            id: base64urlToArrayBuffer(challenge.publicKey.user.id),
          },
          excludeCredentials: challenge.publicKey.excludeCredentials?.map(
            (cred: any) => ({ ...cred, id: base64urlToArrayBuffer(cred.id) })
          ),
        },
      };
      navigator.credentials
        .create(formatted_challenge)
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
          !!user.passkey?.created_at && "hidden"
        )}
        onClick={() => begin_enrollment({})}
      >
        Enroll Passkey 2FA <PlusCircle className="w-4 h-4" />
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
