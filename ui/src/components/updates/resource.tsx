import { useRead } from "@/lib/hooks";
import { getUpdateQuery } from "@/lib/utils";
import { Types } from "komodo_client";
import { useMemo } from "react";
import UpdatesSection from "./section";

export default function ResourceUpdates({ type, id }: Types.ResourceTarget) {
  const deployments = useRead("ListDeployments", {}).data;

  const query = useMemo(
    () => getUpdateQuery({ type, id }, deployments),
    [type, id, deployments],
  );

  // const alerts = useRead("ListAlerts", {
  //   query: getUpdateQuery({ type, id }, deployments),
  // }).data;

  // const openAlerts = alerts?.alerts.filter((alert) => !alert.resolved);

  // const showAlerts = type === "Server";

  return (
    <UpdatesSection query={query} link={`/updates?type=${type}&id=${id}`} />
  );
}
