import { UsableResource } from ".";
import EntityHeader, { EntityHeaderProps } from "@/ui/entity-header";
import { ReactNode } from "react";
import EntityPage, { EntityPageProps } from "@/ui/entity-page";
import { Group, Stack, Text } from "@mantine/core";
import DividedChildren from "@/ui/divided-children";
import ResourceLink from "./link";
import ResourceDescription from "./description";
import { usableResourcePath } from "@/lib/utils";
import ResourceUpdates from "@/components/updates/resource";
import { usePermissions } from "@/lib/hooks";
import Section from "@/ui/section";
import { ICONS } from "@/theme/icons";

export interface ResourceSubPageProps extends EntityHeaderProps {
  parentType: UsableResource;
  parentId: string;
  pageProps?: EntityPageProps;
  entityTypeName?: string;
  info?: ReactNode;
  executions?: ReactNode;
  children?: ReactNode;
}

export default function ResourceSubPage({
  parentType,
  parentId,
  pageProps,
  entityTypeName,
  info,
  executions,
  children,
  ...headerProps
}: ResourceSubPageProps) {
  const { canExecute } = usePermissions({ type: parentType, id: parentId });
  const Header = (
    <Stack justify="space-between">
      <Stack gap="md" pb="md" className="bordered-light" bdrs="md">
        <EntityHeader {...headerProps} />
        <DividedChildren px="md">
          {entityTypeName && <Text>{entityTypeName}</Text>}
          <ResourceLink type={parentType} id={parentId} />
          {info}
        </DividedChildren>
      </Stack>
      <ResourceDescription type={parentType} id={parentId} />
    </Stack>
  );
  return (
    <EntityPage
      {...pageProps}
      backTo={
        pageProps?.backTo ?? `/${usableResourcePath(parentType)}/${parentId}`
      }
    >
      <Stack hiddenFrom="lg" w="100%">
        {Header}
        <ResourceUpdates type={parentType} id={parentId} />
      </Stack>
      <Group
        visibleFrom="lg"
        gap="xl"
        w="100%"
        align="stretch"
        grow
        preventGrowOverflow={false}
      >
        {Header}
        <ResourceUpdates type={parentType} id={parentId} />
      </Group>

      <Stack mt="sm" gap="xl">
        {canExecute && executions && (
          <Section
            title="Execute"
            icon={<ICONS.Execution size="1.3rem" />}
            my="md"
          >
            <Group>{executions}</Group>
          </Section>
        )}

        {children}
      </Stack>
    </EntityPage>
  );
}
