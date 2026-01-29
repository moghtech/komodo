import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { Box, Button, Modal } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { Types } from "komodo_client";
import { MonacoEditor } from "@/components/monaco";
import CopyButton from "@/ui/copy-button";
import LoadingScreen from "@/ui/loading-screen";

export interface ExportTomlProps {
  targets?: Types.ResourceTarget[];
  user_groups?: string[];
  tags?: string[];
  include_variables?: boolean;
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
      >
        Toml
      </Button>
    </>
  );
}

function ExportTomlInner({
  targets,
  user_groups,
  tags,
  include_variables,
}: ExportTomlProps) {
  const useAll = !(targets || user_groups || include_variables);

  const { data: resourcesData, isPending: resourcesPending } = useRead(
    "ExportResourcesToToml",
    {
      targets: targets ? targets : [],
      user_groups: user_groups ? user_groups : [],
      include_variables,
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
