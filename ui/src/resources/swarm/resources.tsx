import { useRead, useUser } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import Section, { SectionProps } from "@/ui/section";
import { Types } from "komodo_client";
import StackTable from "@/resources/stack/table";
import DeploymentTable from "@/resources/deployment/table";
import NewResource from "@/resources/new";

export interface SwarmHostedResourcesSectionProps extends Omit<
  SectionProps,
  "children"
> {
  swarmId: string;
  stacks: Types.StackListItem[];
  deployments: Types.DeploymentListItem[];
}

export default function SwarmHostedResourcesSection({
  swarmId,
  stacks,
  deployments,
  ...sectionProps
}: SwarmHostedResourcesSectionProps) {
  const isAdmin = useUser().data?.admin ?? false;
  const disableNonAdminCreate =
    useRead("GetCoreInfo", {}).data?.disable_non_admin_create ?? true;

  return (
    <Section gap={48} {...sectionProps}>
      <Section
        title="Stacks"
        icon={<ICONS.Stack size="1.3rem" />}
        actions={
          (isAdmin || !disableNonAdminCreate) && (
            <NewResource type="Stack" swarmId={swarmId} />
          )
        }
      >
        <StackTable resources={stacks} />
      </Section>
      <Section
        title="Deployments"
        icon={<ICONS.Deployment size="1.3rem" />}
        actions={
          (isAdmin || !disableNonAdminCreate) && (
            <NewResource type="Deployment" swarmId={swarmId} />
          )
        }
      >
        <DeploymentTable resources={deployments} />
      </Section>
    </Section>
  );
}
