import { ICONS } from "@/lib/icons";
import { Section, SectionProps } from "mogh_ui";
import ResourceTable from "../table";

export interface RepoLinkedResourcesSectionProps extends Omit<
  SectionProps,
  "children"
> {
  repoId: string;
}

export default function RepoLinkedResourcesSection({
  repoId,
  ...sectionProps
}: RepoLinkedResourcesSectionProps) {
  return (
    <Section gap="lg" {...sectionProps}>
      <Section title="Stacks" icon={<ICONS.Stack size="1.3rem" />}>
        <ResourceTable
          type="Stack"
          newProps={{ repoId }}
          specific={{ linked_repos: [repoId] }}
        />
      </Section>
      <Section title="Builds" icon={<ICONS.Build size="1.3rem" />}>
        <ResourceTable
          type="Build"
          newProps={{ repoId }}
          specific={{ linked_repos: [repoId] }}
        />
      </Section>
      <Section title="Syncs" icon={<ICONS.ResourceSync size="1.3rem" />}>
        <ResourceTable
          type="ResourceSync"
          newProps={{ repoId }}
          specific={{ linked_repos: [repoId] }}
        />
      </Section>
    </Section>
  );
}
