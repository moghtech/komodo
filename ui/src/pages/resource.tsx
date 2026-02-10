import ExportToml from "@/components/export-toml";
import ResourceUpdates from "@/components/updates/resource";
import {
  usePermissions,
  usePushRecentlyViewed,
  useRead,
  useResourceParamType,
  useSetTitle,
} from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import {
  ResourceComponents,
  SETTINGS_RESOURCES,
  UsableResource,
} from "@/resources";
import { AddResourceTags, ResourceTags } from "@/resources/tags";
import DividedChildren from "@/ui/divided-children";
import Section from "@/ui/section";
import { Anchor, Group, Stack, Text } from "@mantine/core";
import { Types } from "komodo_client";
import { useParams } from "react-router-dom";
import EntityPage from "@/ui/entity-page";
import { usableResourcePath } from "@/lib/utils";
import ResourceDescription from "@/resources/description";
import ResourceNotFound from "@/resources/not-found";

export default function Resource() {
  const type = useResourceParamType()!;
  const id = useParams().id as string;

  if (!type || !id) return null;

  return <ResourceInner type={type} id={id} />;
}

function ResourceInner({ type, id }: { type: UsableResource; id: string }) {
  const Components = ResourceComponents[type];
  const resources = useRead(`List${type}s`, {}).data;
  const resource = Components.useListItem(id);

  const { canExecute } = usePermissions({ type, id });

  usePushRecentlyViewed({ type, id });
  useSetTitle(resource?.name);

  if (!type || !id) return null;

  if (!resource) {
    if (resources) return <ResourceNotFound type={type} />;
    else return null;
  }

  let showExport = true;
  if (type === "ResourceSync") {
    const info = resource?.info as Types.ResourceSyncListItemInfo;
    showExport = !info?.file_contents && (info.file_contents || !info.managed);
  }

  return (
    <EntityPage
      backTo={
        "/" +
        (SETTINGS_RESOURCES.includes(type)
          ? "settings"
          : usableResourcePath(type))
      }
      actions={
        <>
          {/* {type !== "Server" && canCreate && (
            <CopyResource type={type} id={id} />
          )} */}
          {showExport && <ExportToml targets={[{ type, id }]} />}
        </>
      }
    >
      <Stack hiddenFrom="xl2" w="100%">
        <ResourceHeader type={type} id={id} />
        <ResourceUpdates type={type} id={id} />
      </Stack>
      <Group
        visibleFrom="xl2"
        gap="xl"
        w="100%"
        align="stretch"
        grow
        preventGrowOverflow={false}
      >
        <ResourceHeader type={type} id={id} />
        <ResourceUpdates type={type} id={id} />
      </Group>

      <Stack mt="lg" gap="xl">
        {canExecute && Object.keys(Components.Executions).length > 0 && (
          <Section
            title="Execute"
            icon={<ICONS.Execution size="1rem" />}
            my="xl"
          >
            <Group>
              {Object.entries(Components.Executions).map(([key, Action]) => (
                <Action key={key} id={id} />
              ))}
            </Group>
          </Section>
        )}
        {Object.entries(Components.Page).map(([key, Component]) => (
          <Component key={key} id={id} />
        ))}
        <Components.Config id={id} />
      </Stack>
    </EntityPage>
  );
}

function ResourceHeader({ type, id }: { type: UsableResource; id: string }) {
  const Components = ResourceComponents[type];
  const resource = Components.useListItem(id);
  const links = Components.useResourceLinks(resource);
  const { canWrite } = usePermissions({ type, id });

  const infoEntries = Object.entries(Components.Info);

  return (
    <Stack justify="space-between">
      <Stack
        gap="md"
        pb="md"
        bd="1px solid var(--mantine-color-accent-border-0)"
        bdrs="md"
      >
        <Components.ResourcePageHeader id={id} />
        {infoEntries.length > 0 && (
          <DividedChildren px="md">
            {infoEntries.map(([key, Info]) => (
              <Info key={key} id={id} />
            ))}
          </DividedChildren>
        )}
        {links && links.length > 0 && (
          <Group px="md">
            {links.map((link) => (
              <Anchor
                key={link}
                target="_blank"
                href={link}
                style={{
                  display: "flex",
                  alignItems: "center",
                }}
              >
                <ICONS.Link size="1rem" />
                <Text hiddenFrom="lg" maw={150} truncate>
                  {link}
                </Text>
                <Text visibleFrom="lg" maw={250} truncate>
                  {link}
                </Text>
              </Anchor>
            ))}
          </Group>
        )}
        <Group px="md" gap="sm">
          {!resource?.tags.length && (
            <Text c="dimmed" fz="sm">
              Tags:
            </Text>
          )}
          <ResourceTags
            target={{ id, type }}
            disabled={!canWrite}
            click_to_delete
          />
          {canWrite && <AddResourceTags id={id} type={type} />}
        </Group>
      </Stack>
      <ResourceDescription type={type} id={id} />
    </Stack>
  );
}
