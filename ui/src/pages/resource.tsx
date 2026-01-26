import ResourceUpdates from "@/components/updates/resource";
import {
  usePermissions,
  usePushRecentlyViewed,
  useRead,
  useResourceParamType,
  useSetTitle,
} from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { usableResourcePath } from "@/lib/utils";
import {
  ResourceComponents,
  SETTINGS_RESOURCES,
  UsableResource,
} from "@/resources";
import {
  ResourceDescription,
  ResourceNotFound,
  ResourceTags,
} from "@/resources/common";
import { Anchor, Box, Button, Flex, Group, Stack, Text } from "@mantine/core";
import { Types } from "komodo_client";
import { Link, useParams } from "react-router-dom";

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

  usePushRecentlyViewed({ type, id });
  useSetTitle(resource?.name);

  // const { canCreate, canExecute, canWrite } = usePermissions({ type, id });

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
    <Stack gap="lg">
      <Flex align="center" justify="space-between" mb="lg">
        <Button
          variant="light"
          c="inherit"
          bg="accent"
          leftSection={<ICONS.Back className="w-4" />}
          renderRoot={(props) => (
            <Link
              to={
                "/" +
                (SETTINGS_RESOURCES.includes(type)
                  ? "settings"
                  : usableResourcePath(type))
              }
              {...props}
            />
          )}
        >
          Back
        </Button>

        <Group>
          {/* {type !== "Server" && canCreate && (
            <CopyResource type={type} id={id} />
          )}
          {showExport && <ExportButton targets={[{ type, id }]} />} */}
        </Group>
      </Flex>

      <Stack hiddenFrom="xl" w="100%">
        <ResourceHeader type={type} id={id} />
        <ResourceUpdates type={type} id={id} />
      </Stack>
      <Group
        visibleFrom="xl"
        gap="xl"
        w="100%"
        align="stretch"
        grow
        preventGrowOverflow={false}
      >
        <ResourceHeader type={type} id={id} />
        <ResourceUpdates type={type} id={id} />
      </Group>

      <Stack mt="lg">
        {/* {canExecute && Object.keys(Components.Actions).length > 0 && (
            <Section title="Execute" icon={<Zap className="w-4 h-4" />}>
              <div className="flex gap-4 items-center flex-wrap">
                {Object.entries(Components.Actions).map(([key, Action]) => (
                  <Action key={key} id={id} />
                ))}
              </div>
            </Section>
          )} */}
        {Object.entries(Components.Page).map(([key, Component]) => (
          <Component key={key} id={id} />
        ))}
        <Components.Config id={id} />
        {/* {canWrite && (
            <Section
              title="Danger Zone"
              icon={<AlertTriangle className="w-6 h-6" />}
            >
              <Components.DangerZone id={id} />
            </Section>
          )} */}
      </Stack>
    </Stack>
  );
}

function ResourceHeader({ type, id }: { type: UsableResource; id: string }) {
  const Components = ResourceComponents[type];
  const resource = Components.useListItem(id);
  const links = Components.useResourceLinks(resource);
  const { canWrite } = usePermissions({ type, id });

  const infoEntries = Object.entries(Components.Info);
  const statusEntries = Object.entries(Components.Status);

  return (
    <Stack justify="space-between">
      <Stack gap={0}>
        <Components.ResourcePageHeader id={id} />
        <Group>
          {infoEntries.map(([key, Info]) => (
            <Box
              key={key}
              pr="md"
              fz="sm"
              bdrs="md"
              style={{
                borderRight: "1px solid var(--mantine-color-accent-border-0)",
              }}
            >
              <Info id={id} />
            </Box>
          ))}
          {statusEntries.map(([key, Status]) => (
            <Status key={key} id={id} />
          ))}
        </Group>
        {links && links.length > 0 && (
          <Group>
            {links.map((link) => (
              <Anchor
                key={link}
                target="_blank"
                href={link}
                style={{ display: "flex", alignItems: "center" }}
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
        <Group
          h="fit-content"
          px="lg"
          py="sm"
          style={{
            borderWidth: 1,
            borderTopWidth: 0,
            borderStyle: "solid",
            borderColor: "var(--mantine-color-accent-border-0)",
            borderBottomLeftRadius: "var(--mantine-radius-md)",
            borderBottomRightRadius: "var(--mantine-radius-md)",
          }}
        >
          <Text c="dimmed" fz="sm">
            Tags:
          </Text>
          <ResourceTags
            target={{ id, type }}
            disabled={!canWrite}
            click_to_delete
          />
          {/* {canWrite && <AddTags target={{ id, type }} />} */}
        </Group>
      </Stack>
      <ResourceDescription type={type} id={id} />
    </Stack>
  );
}
