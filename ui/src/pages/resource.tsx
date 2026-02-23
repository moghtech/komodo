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
import { Group, Stack, Text } from "@mantine/core";
import { Types } from "komodo_client";
import { Link, useParams } from "react-router-dom";
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
  const RC = ResourceComponents[type];
  const resources = useRead(`List${type}s`, {}).data;
  const resource = RC.useListItem(id);

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
      <Stack hiddenFrom="lg" w="100%">
        <ResourceHeader type={type} id={id} />
        <ResourceUpdates type={type} id={id} />
      </Stack>
      <Group
        visibleFrom="lg"
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
        {canExecute && Object.keys(RC.Executions).length > 0 && (
          <Section
            title="Execute"
            icon={<ICONS.Execution size="1rem" />}
            my="xl"
          >
            <Group>
              {Object.entries(RC.Executions).map(([key, Action]) => (
                <Action key={key} id={id} />
              ))}
            </Group>
          </Section>
        )}
        {Object.entries(RC.Page).map(([key, Component]) => (
          <Component key={key} id={id} />
        ))}
        <RC.Config id={id} />
      </Stack>
    </EntityPage>
  );
}

function ResourceHeader({ type, id }: { type: UsableResource; id: string }) {
  const RC = ResourceComponents[type];
  const resource = RC.useFull(id);
  const links = RC.useResourceLinks(resource);
  const { canWrite } = usePermissions({ type, id });

  const infoEntries = Object.entries(RC.Info);

  return (
    <Stack justify="space-between">
      <Stack gap="md" pb="md" className="bordered-light" bdrs="md">
        <RC.ResourcePageHeader id={id} />
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
              <Group
                key={link}
                renderRoot={(props) => (
                  <Link target="_blank" to={link} {...props} />
                )}
                gap="xs"
              >
                <ICONS.Link size="1rem" />
                <Text
                  className="hover-underline"
                  hiddenFrom="lg"
                  maw={150}
                  truncate
                >
                  {link}
                </Text>
                <Text
                  className="hover-underline"
                  visibleFrom="lg"
                  maw={250}
                  truncate
                >
                  {link}
                </Text>
              </Group>
            ))}
          </Group>
        )}
        <Group px="md" gap="sm">
          {!resource?.tags?.length && (
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
