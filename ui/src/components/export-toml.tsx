import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { Box, Button, Modal } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { Types } from "komodo_client";
import { MonacoEditor } from "@/components/monaco";
import { CopyButton } from "mogh_ui";
import { LoadingScreen } from "mogh_ui";

export interface ExportTomlProps {
  targets?: Types.ResourceTarget[];
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
  userGroups,
  tags,
  includeVariables,
}: ExportTomlProps) {
  const useAll = !(targets || userGroups || includeVariables);

  const { data: resourcesData, isPending: resourcesPending } = useRead(
    "ExportResourcesToToml",
    {
      targets: targets ? targets : [],
      user_groups: userGroups ? userGroups : [],
      include_variables: includeVariables,
    },
    { enabled: !useAll },
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
      <MonacoEditor value={data?.toml} language="fancy_toml" readOnly />
      <Box pos="absolute" top={18} right={18} style={{ zIndex: 10 }}>
        <CopyButton content={data?.toml ?? ""} />
      </Box>
    </Box>
  );
}
