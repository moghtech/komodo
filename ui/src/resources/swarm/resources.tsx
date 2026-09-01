import { ICONS } from "@/lib/icons";
import { Section, SectionProps } from "mogh_ui";
import ResourceTable from "../table";

export interface SwarmHostedResourcesSectionProps extends Omit<
  SectionProps,
  "children"
> {
  swarmId: string;
}

export default function SwarmHostedResourcesSection({
  swarmId,
  ...sectionProps
}: SwarmHostedResourcesSectionProps) {
  return (
    <Section gap={48} {...sectionProps}>
      <Section title="Stacks" icon={<ICONS.Stack size="1.3rem" />}>
        <ResourceTable
          type="Stack"
          newProps={{ swarmId }}
          specific={{ swarm_ids: [swarmId] }}
        />
      </Section>
      <Section title="Deployments" icon={<ICONS.Deployment size="1.3rem" />}>
        <ResourceTable
          type="Deployment"
          newProps={{ swarmId }}
          specific={{ swarm_ids: [swarmId] }}
        />
      </Section>
    </Section>
  );
}
