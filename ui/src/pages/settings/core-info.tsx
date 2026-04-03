import { MonacoEditor } from "@/components/monaco";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import CopyText from "@/ui/copy-text";
import DividedChildren from "@/ui/divided-children";
import {
  ActionIcon,
  Box,
  Center,
  Group,
  Loader,
  Modal,
  Text,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";

export default function SettingsCoreInfo() {
  const info = useRead("GetCoreInfo", {}).data;
  return (
    <DividedChildren>
      <Box>
        <Text
          ff="monospace"
          fz="xl"
          className="accent-hover-light bordered-heavy"
          bdrs="sm"
          px="lg"
          py="0.5rem"
        >
          {info?.title}
        </Text>
      </Box>

      <AllInfo />

      {info?.public_key && (
        <Group gap="xs">
          <Text c="dimmed" size="lg">
            Public Key:
          </Text>
          <CopyText
            content={info.public_key}
            label="public key"
            ff="monospace"
            className="text-ellipsis accent-hover-light bordered-heavy"
          />
        </Group>
      )}
    </DividedChildren>
  );
}

function AllInfo() {
  const [opened, { open, close }] = useDisclosure();
  const info = useRead("GetCoreInfo", {}).data;
  return (
    <Box>
      <Modal
        opened={opened}
        onClose={close}
        title={<Text size="xl">Core Info</Text>}
        size="xl"
      >
        {info ? (
          <MonacoEditor
            value={JSON.stringify(info, undefined, 2)}
            language="json"
            readOnly
          />
        ) : (
          <Center h="20vh">
            <Loader size="xl" />
          </Center>
        )}
      </Modal>

      <ActionIcon onClick={open} size="lg">
        <ICONS.Info size="1rem" />
      </ActionIcon>
    </Box>
  );
}
