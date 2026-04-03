import { ICONS } from "@/lib/icons";
import { Types } from "komodo_client";
import Section, { SectionProps } from "@/ui/section";
import StackTable from "@/resources/stack/table";
import BuildTable from "../build/table";
import ResourceSyncTable from "../sync/table";

export interface RepoLinkedResourcesSectionProps extends Omit<
  SectionProps,
  "children"
> {
  stacks: Types.StackListItem[];
  builds: Types.BuildListItem[];
  syncs: Types.ResourceSyncListItem[];
}

export default function RepoLinkedResourcesSection({
  stacks,
  builds,
  syncs,
  ...sectionProps
}: RepoLinkedResourcesSectionProps) {
  return (
    <Section gap={48} {...sectionProps}>
      {stacks.length ? (
        <Section title="Stacks" icon={<ICONS.Stack size="1.3rem" />}>
          <StackTable resources={stacks} />
        </Section>
      ) : null}
      {builds.length ? (
        <Section title="Builds" icon={<ICONS.Build size="1.3rem" />}>
          <BuildTable resources={builds} />
        </Section>
      ) : null}
      {syncs.length ? (
        <Section title="Syncs" icon={<ICONS.ResourceSync size="1.3rem" />}>
          <ResourceSyncTable resources={syncs} />
        </Section>
      ) : null}
    </Section>
  );
}
