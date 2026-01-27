import { useRead } from "@/lib/hooks";
import { getUpdateQuery } from "@/lib/utils";

export default function ActionInfo({ id }: { id: string }) {
  const update = useRead("ListUpdates", {
    query: {
      ...getUpdateQuery({ type: "Action", id }, undefined),
      operation: "RunAction",
    },
  }).data?.updates[0];

  const full_update = useRead(
    "GetUpdate",
    { id: update?.id! },
    { enabled: !!update?.id },
  ).data;

  const log = full_update?.logs.find((log) => log.stage === "Execute Action");

  return <>ACTION INFO</>;
}
