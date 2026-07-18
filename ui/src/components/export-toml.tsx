import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { UsableResource } from "@/resources";
import { Box, Button, Modal } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { Types } from "komodo_client";
import { MonacoEditor } from "mogh_ui";
import { CopyButton } from "mogh_ui";
import { LoadingScreen } from "mogh_ui";

export interface ExportTomlProps {
  targets?: Types.ResourceTarget[];
  /**
   * Export all resources of the type matching the query,
   * not just the ones on the current page.
   * The matching targets are only fetched when the modal is opened.
   */
  listQuery?: {
    type: UsableResource;
    query: Types.ResourceQuery<any>;
  };
  userGroups?: string[];
  tags?: string[];
  includeVariables?: boolean;
}

export default function ExportToml(props: ExportTomlProps) {
  const [opened, { open, close }] = useDisclosure();

  return (
    <>
      <Modal opened={opened} onClose={close} title="Export to Toml" size="auto">
        {opened && <ExportTomlInner {...props} />}
      </Modal>

      <Button
        variant="default"
        leftSection={<ICONS.ExportToml size="1.1rem" />}
        onClick={open}
        w={{ base: "100%", xs: "fit-content" }}
      >
        Toml
      </Button>
    </>
  );
}

function ExportTomlInner({
  targets,
  listQuery,
  userGroups,
  tags,
  includeVariables,
}: ExportTomlProps) {
  const useAll = !(targets || listQuery || userGroups || includeVariables);

  const listTargets = useRead(
    `List${listQuery?.type ?? "Server"}s`,
    { query: listQuery?.query, limit: 0 },
    { enabled: !!listQuery },
  ).data?.map(
    (resource) =>
      ({ type: listQuery!.type, id: resource.id }) as Types.ResourceTarget,
  );

  const exportTargets = listQuery ? listTargets : targets;

  const { data: resourcesData, isPending: resourcesPending } = useRead(
    "ExportResourcesToToml",
    {
      targets: exportTargets ? exportTargets : [],
      user_groups: userGroups ? userGroups : [],
      include_variables: includeVariables,
    },
    { enabled: !useAll && (!listQuery || !!listTargets) },
  );

  const { data: allData, isPending: allPending } = useRead(
    "ExportAllResourcesToToml",
    {
      tags,
      include_resources: true,
      include_variables: true,
      include_user_groups: true,
    },
    { enabled: useAll },
  );

  const [data, loading] = useAll
    ? [allData, allPending]
    : [resourcesData, resourcesPending];

  const enableFancyToml = useRead("GetCoreInfo", {}).data?.enable_fancy_toml;

  return (
    <Box
      pos="relative"
      w={{
        base: "calc(100vw - 5rem)",
        xs: "calc(100vw - 8rem)",
        md: "calc(100vw - 12rem)",
      }}
      maw={1200}
    >
      {loading && <LoadingScreen mt="0" h="30vh" />}
      <MonacoEditor
        value={data?.toml}
        language="fancy_toml"
        enableFancyToml={enableFancyToml}
        readOnly
      />
      <Box pos="absolute" top={18} right={18} style={{ zIndex: 10 }}>
        <CopyButton content={data?.toml ?? ""} />
      </Box>
    </Box>
  );
}
