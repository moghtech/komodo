import { AUTH_TOKEN_STORAGE_KEY } from "@main";
import { Button } from "@ui/button";
import { UserX } from "lucide-react";

export default function UserDisabled() {
  return (
    <div className="w-full h-screen flex justify-center items-center">
      <div className="flex flex-col gap-4 justify-center items-center">
        <UserX className="w-16 h-16" />
        User Not Enabled
        <Button
          variant="outline"
          onClick={() => {
            localStorage.removeItem(AUTH_TOKEN_STORAGE_KEY);
            location.reload();
          }}
        >
          Log Out
        </Button>
      </div>
    </div>
  );
}
