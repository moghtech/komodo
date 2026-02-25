import { alertLevelIntention } from "@/lib/color";
import StatusBadge from "@/ui/status-badge";
import { Types } from "komodo_client";

export default function AlertLevel({
  level,
}: {
  level: Types.SeverityLevel | undefined;
}) {
  if (!level) return null;
  return <StatusBadge text={level} intent={alertLevelIntention(level)} />;
}
