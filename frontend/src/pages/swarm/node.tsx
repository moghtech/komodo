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

export default function SwarmNodePage() {
  const { id, node: __node } = useParams() as {
    id: string;
    node: string;
  };
  const _node = decodeURIComponent(__node);
  const swarm = useSwarm(id);
  const { data: node, isPending } = useRead("InspectSwarmNode", {
    swarm: id,
    node: _node,
  });
  useSetTitle(
    `${swarm?.name} | Node | ${node?.Spec?.Name ?? node?.Description?.Hostname ?? node?.ID ?? "Unknown"}`
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

  if (!node) {
    return <div className="flex w-full py-4">Failed to inspect node.</div>;
  }

  const Icon = SWARM_ICONS.Node;

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
          <PageHeaderName
            name={node.Spec?.Name ?? node.Description?.Hostname ?? node.ID}
          />
        </div>

        {/* INFO */}
        <div className="flex flex-wrap gap-4 items-center text-muted-foreground">
          Swarm Node
          <ResourceLink type="Swarm" id={id} />
        </div>
      </div>

      {canExecute && node.ID && (
        <Section title="Execute" icon={<Zap className="w-4 h-4" />}>
          <div className="flex gap-4 items-center flex-wrap">
            <RemoveSwarmResource
              id={id}
              type="Node"
              resource_id={node.ID}
              resource_name={node.Description?.Hostname}
            />
          </div>
        </Section>
      )}

      <MonacoEditor
        value={JSON.stringify(node, null, 2)}
        language="json"
        readOnly
      />
    </div>
  );
}
