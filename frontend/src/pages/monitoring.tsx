import { Page } from "@components/layouts";
import { useSetTitle } from "@lib/hooks";
import { LineChart } from "lucide-react";

export default function MonitoringPage() {
  useSetTitle("Monitoring");
  return (
    <Page title="Monitoring" icon={<LineChart className="w-8 h-8" />}>
      <div />
    </Page>
  );
}


