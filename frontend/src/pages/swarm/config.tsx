import { ResourceLink } from "@components/resources/common";
import { PageHeaderName } from "@components/util";
import { usePermissions, useRead, useSetTitle } from "@lib/hooks";
import { Button } from "@ui/button";
import { ChevronLeft, Loader2, Zap } from "lucide-react";
import { useNavigate, useParams } from "react-router-dom";
import { MonacoEditor } from "@components/monaco";
import { SWARM_ICONS, useSwarm } from "@components/resources/swarm";
import { Section } from "@components/layouts";
import { RemoveSwarmResource } from "./remove";

export default function SwarmConfigPage() {
  const { id, config: __config } = useParams() as {
    id: string;
    config: string;
  };
  const _config = decodeURIComponent(__config);
  const swarm = useSwarm(id);
  const { data, isPending } = useRead("InspectSwarmConfig", {
    swarm: id,
    config: _config,
  });
  const config = data?.[0];
  useSetTitle(
    `${swarm?.name} | Config | ${config?.Spec?.Name ?? config?.ID ?? "Unknown"}`
  );
  const { canExecute } = usePermissions({
    type: "Swarm",
    id,
  });
  const nav = useNavigate();

  if (isPending) {
    return (
      <div className="flex justify-center w-full py-4">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  if (!config) {
    return <div className="flex w-full py-4">Failed to inspect config.</div>;
  }

  const Icon = SWARM_ICONS.Config;

  return (
    <div className="flex flex-col gap-16 mb-24">
      {/* HEADER */}
      <div className="flex flex-col gap-4">
        {/* BACK */}
        <div className="flex items-center justify-between mb-4">
          <Button
            className="gap-2"
            variant="secondary"
            onClick={() => nav("/swarms/" + id)}
          >
            <ChevronLeft className="w-4" /> Back
          </Button>
        </div>

        {/* TITLE */}
        <div className="flex items-center gap-4">
          <div className="mt-1">
            <Icon size={8} />
          </div>
          <PageHeaderName name={config?.Spec?.Name ?? config?.ID} />
        </div>

        {/* INFO */}
        <div className="flex flex-wrap gap-4 items-center text-muted-foreground">
          Swarm Config
          <ResourceLink type="Swarm" id={id} />
        </div>
      </div>

      {canExecute && config.ID && (
        <Section title="Execute" icon={<Zap className="w-4 h-4" />}>
          <div className="flex gap-4 items-center flex-wrap">
            <RemoveSwarmResource
              id={id}
              type="Config"
              resource_id={config.ID}
              resource_name={config.Spec?.Name}
            />
          </div>
        </Section>
      )}

      <MonacoEditor
        value={JSON.stringify(config, null, 2)}
        language="json"
        readOnly
      />
    </div>
  );
}
