import { ResourceLink } from "@components/resources/common";
import { PageHeaderName } from "@components/util";
import { usePermissions, useRead, useSetTitle } from "@lib/hooks";
import { Button } from "@ui/button";
import { ChevronLeft, Loader2, Zap } from "lucide-react";
import { useNavigate, useParams } from "react-router-dom";
import { MonacoEditor } from "@components/monaco";
import { SWARM_ICONS, useSwarm } from "@components/resources/swarm";
import { RemoveSwarmResource } from "./remove";
import { Section } from "@components/layouts";

export default function SwarmSecretPage() {
  const { id, secret: __secret } = useParams() as {
    id: string;
    secret: string;
  };
  const _secret = decodeURIComponent(__secret);
  const swarm = useSwarm(id);
  const { data: secret, isPending } = useRead("InspectSwarmSecret", {
    swarm: id,
    secret: _secret,
  });
  useSetTitle(
    `${swarm?.name} | Secret | ${secret?.Spec?.Name ?? secret?.ID ?? "Unknown"}`
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

  if (!secret) {
    return <div className="flex w-full py-4">Failed to inspect secret.</div>;
  }

  const Icon = SWARM_ICONS.Secret;

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
          <PageHeaderName name={secret?.Spec?.Name ?? secret?.ID} />
        </div>

        {/* INFO */}
        <div className="flex flex-wrap gap-4 items-center text-muted-foreground">
          Swarm Secret
          <ResourceLink type="Swarm" id={id} />
        </div>
      </div>

      {canExecute && secret.ID && (
        <Section title="Execute" icon={<Zap className="w-4 h-4" />}>
          <div className="flex gap-4 items-center flex-wrap">
            <RemoveSwarmResource
              id={id}
              type="Secret"
              resource_id={secret.ID}
              resource_name={secret.Spec?.Name}
            />
          </div>
        </Section>
      )}

      <MonacoEditor
        value={JSON.stringify(secret, null, 2)}
        language="json"
        readOnly
      />
    </div>
  );
}
