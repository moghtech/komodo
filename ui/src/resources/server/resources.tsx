import { ICONS } from "@/lib/icons";
import { Section, SectionProps } from "mogh_ui";
import ResourceTable from "../table";

export interface ServerHostedResourcesSectionProps extends Omit<
  SectionProps,
  "children"
> {
  serverId: string;
}

export default function ServerHostedResourcesSection({
  serverId,
  ...sectionProps
}: ServerHostedResourcesSectionProps) {
  return (
    <Section gap={48} {...sectionProps}>
      <Section title="Stacks" icon={<ICONS.Stack size="1.3rem" />}>
        <ResourceTable
          type="Stack"
          newProps={{ serverId }}
          specific={{ server_ids: [serverId] }}
        />
      </Section>
      <Section title="Deployments" icon={<ICONS.Deployment size="1.3rem" />}>
        <ResourceTable
          type="Deployment"
          newProps={{ serverId }}
          specific={{ server_ids: [serverId] }}
        />
      </Section>
      <Section title="Repos" icon={<ICONS.Repo size="1.3rem" />}>
        <ResourceTable
          type="Repo"
          newProps={{ serverId }}
          specific={{ server_ids: [serverId] }}
        />
      </Section>
    </Section>
  );
}
