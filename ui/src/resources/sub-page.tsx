import { ResourceComponents, UsableResource } from ".";
import {
  EntityHeader,
  EntityHeaderProps,
  EntityPage,
  EntityPageProps,
  PageBreadcrumbs,
} from "mogh_ui";
import { ReactNode } from "react";
import { Group, Stack, Text } from "@mantine/core";
import { DividedChildren } from "mogh_ui";
import ResourceLink from "./link";
import ResourceDescription from "./description";
import { resourceTypeCrumb, usableResourcePath } from "@/lib/utils";
import ResourceUpdates from "@/components/updates/resource";
import { usePermissions } from "@/lib/hooks";
import { Section } from "mogh_ui";
import { ICONS } from "@/lib/icons";

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
  const parent = ResourceComponents[parentType].useListItem(parentId);
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
      breadcrumbs={
        pageProps?.breadcrumbs ?? (
          <PageBreadcrumbs
            items={[
              resourceTypeCrumb(parentType),
              {
                label: parent?.name ?? "Unknown",
                to: `/${usableResourcePath(parentType)}/${parentId}`,
              },
              { label: headerProps.name },
            ]}
          />
        )
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

      <Stack gap="xl">
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
