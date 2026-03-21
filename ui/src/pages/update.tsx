import UpdateDetails from "@/components/updates/details";
import { useRead, useSetTitle } from "@/lib/hooks";
import PageGuard from "@/ui/page-guard";
import { useParams } from "react-router-dom";

export default function Update() {
  useSetTitle("Update");
  const id = useParams().id as string;
  const { data: update, isPending } = useRead(
    "GetUpdate",
    { id },
    { refetchInterval: 10_000 },
  );

  return (
    <PageGuard
      isPending={isPending}
      error={!update ? "Update could not be found." : undefined}
    >
      <UpdateDetails updateId={id} />
    </PageGuard>
  );
}
