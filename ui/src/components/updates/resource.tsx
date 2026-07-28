import { useRead } from "@/lib/hooks";
import { getUpdateQuery } from "@/lib/utils";
import { Types } from "komodo_client";
import { useMemo } from "react";
import UpdatesSection from "./section";

export default function ResourceUpdates({ type, id }: Types.ResourceTarget) {
  const deployment = useRead(
    "GetDeployment",
    { deployment: id },
    { enabled: type === "Deployment" },
  ).data;
  const buildId =
    deployment?.config?.image?.type === "Build"
      ? deployment.config.image.params.build_id
      : undefined;

  const query = useMemo(
    () => getUpdateQuery({ type, id }, buildId),
    [type, id],
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
