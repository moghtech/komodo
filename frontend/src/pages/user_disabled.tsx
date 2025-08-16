import { LOGIN_TOKENS, useUser } from "@lib/hooks";
import { Button } from "@ui/button";
import { UserX } from "lucide-react";

export default function UserDisabled() {
  const user_id = useUser().data?._id?.$oid;
  return (
    <div className="w-full h-screen flex justify-center items-center">
      <div className="flex flex-col gap-4 justify-center items-center">
        <UserX className="w-16 h-16" />
        User Not Enabled
        <Button
          variant="outline"
          onClick={() => {
            user_id && LOGIN_TOKENS.remove(user_id);
            location.reload();
          }}
        >
          Log Out
        </Button>
      </div>
    </div>
  );
}
