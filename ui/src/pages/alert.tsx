import AlertDetails from "@/components/alerts/details";
import { useRead, useSetTitle } from "@/lib/hooks";
import { PageGuard } from "mogh_ui";
import { useParams } from "react-router-dom";

export default function Alert() {
  useSetTitle("Alert");
  const id = useParams().id as string;
  const { data: alert, isPending } = useRead("GetAlert", { id });

  return (
    <PageGuard
      isPending={isPending}
      error={!alert ? "Alert could not be found." : undefined}
    >
      <AlertDetails alertId={id} />
    </PageGuard>
  );
}
