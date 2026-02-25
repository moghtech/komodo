import { useRead, useUser } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { Types } from "komodo_client";
import Section, { SectionProps } from "@/ui/section";
import StackTable from "@/resources/stack/table";
import DeploymentTable from "@/resources/deployment/table";
import RepoTable from "@/resources/repo/table";
import NewResource from "@/resources/new";
import { ResourceComponents } from "..";

export interface ServerHostedResourcesSectionProps extends Omit<
  SectionProps,
  "children"
> {
  serverId: string;
  stacks: Types.StackListItem[];
  deployments: Types.DeploymentListItem[];
  repos: Types.RepoListItem[];
}

export default function ServerHostedResourcesSection({
  serverId,
  stacks,
  deployments,
  repos,
  ...sectionProps
}: ServerHostedResourcesSectionProps) {
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
            <ResourceComponents.Stack.New serverId={serverId} />
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
            <ResourceComponents.Deployment.New serverId={serverId} />
          )
        }
      >
        <DeploymentTable resources={deployments} />
      </Section>
      <Section
        title="Repos"
        icon={<ICONS.Repo size="1.3rem" />}
        actions={
          (isAdmin || !disableNonAdminCreate) && (
            <ResourceComponents.Repo.New serverId={serverId} />
          )
        }
      >
        <RepoTable resources={repos} />
      </Section>
    </Section>
  );
}
