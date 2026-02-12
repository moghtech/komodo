import { MonacoEditor } from "@/components/monaco";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import CopyButton from "@/ui/copy-button";
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
          bg="accent.3"
          bd="1px solid var(--mantine-color-accent-border-4)"
          bdrs="md"
          px="lg"
          py="0.5rem"
        >
          {info?.title}
        </Text>
      </Box>

      <AllInfo />

      {info?.public_key && (
        <Group gap="xs">
          <Text c="dimmed" size="lg">Public Key:</Text>
          <Text
            ff="monospace"
            title={info?.public_key}
            className="text-ellipsis"
            w={{ base: 150, md: 230 }}
            bg="accent.3"
            bd="1px solid var(--mantine-color-accent-border-4)"
            bdrs="md"
            px="md"
            py="0.4rem"
          >
            {info?.public_key}
          </Text>
          <CopyButton content={info.public_key} />
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
