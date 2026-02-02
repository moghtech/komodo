import { useTags, useWrite } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import ConfirmButton from "@/ui/confirm-button";
import { notifications } from "@mantine/notifications";

export default function BatchDeleteAllTerminals({
  refetch,
  noTerminals,
}: {
  refetch: () => void;
  noTerminals: boolean;
}) {
  const { mutate, isPending } = useWrite("BatchDeleteAllTerminals", {
    onSuccess: () => {
      refetch();
      notifications.show({ message: "Deleted All Terminals" });
    },
  });
  const { tags } = useTags();
  return (
    <ConfirmButton
      color="red"
      icon={<ICONS.Delete className="w-4 h-4" />}
      w={160}
      onClick={() => mutate({ query: { tags } })}
      disabled={noTerminals}
      loading={isPending}
    >
      Delete All
    </ConfirmButton>
  );
}
