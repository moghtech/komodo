import { ICONS } from "@/lib/icons";
import { Section, SectionProps, useManageAuth } from "mogh_ui";
import NewApiKey from "./new";
import ApiKeysTable from "./table";
import { useInvalidate, useRead, useWrite } from "@/lib/hooks";
import { notifications } from "@mantine/notifications";
import { Box, Button, Group, Modal, Stack, Text } from "@mantine/core";
import { CopyText } from "mogh_ui";
import { Types } from "komodo_client";
import { useState } from "react";

export interface ApiKeysSectionProps extends SectionProps {
  /** For service user api keys */
  userId?: string;
}

export default function ApiKeysSection({
  userId,
  ...sectionProps
}: ApiKeysSectionProps) {
  const { data: keys, isPending } = useRead(
    userId ? "ListApiKeysForServiceUser" : "ListApiKeys",
    userId
      ? {
          user: userId,
        }
      : {},
  );
  const inv = useInvalidate();
  const { mutate: regularDelete, isPending: regularPending } = useManageAuth(
    "DeleteApiKey",
    {
      onSuccess: () => {
        inv(["ListApiKeys"]);
        notifications.show({ message: "API key deleted.", color: "green" });
      },
    },
  );
  const { mutate: serviceDelete, isPending: servicePending } = useWrite(
    "DeleteApiKeyForServiceUser",
    {
      onSuccess: () => {
        inv(["ListApiKeysForServiceUser"]);
        notifications.show({ message: "API key deleted.", color: "green" });
      },
    },
  );
  const [rotated, setRotated] = useState<Types.RotateApiKeyResponse>();
  const { mutate: rotate, isPending: rotatePending } = useWrite(
    "RotateApiKey",
    {
      onSuccess: (res) => {
        inv([userId ? "ListApiKeysForServiceUser" : "ListApiKeys"]);
        setRotated(res);
      },
    },
  );
  return (
    <Section
      isPending={isPending}
      title="API Keys"
      titleFz="h3"
      icon={<ICONS.Key size="1.2rem" />}
      titleRight={
        <Box ml="md">
          <NewApiKey userId={userId} />
        </Box>
      }
      withBorder
      {...sectionProps}
    >
      {keys && (
        <ApiKeysTable
          noBorder
          keys={keys}
          onRotate={(key) => rotate({ key })}
          rotatePending={rotatePending}
          onDelete={(key) =>
            userId ? serviceDelete({ key }) : regularDelete({ key })
          }
          deletePending={userId ? servicePending : regularPending}
        />
      )}

      <Modal
        opened={!!rotated}
        onClose={() => setRotated(undefined)}
        title={<Text size="lg">API Key Rotated</Text>}
      >
        {rotated && (
          <Stack>
            <Text>
              Copy the new API key and secret.{" "}
              <b>The secret will not be shown again.</b>
            </Text>

            <Group justify="space-between" wrap="nowrap">
              <Text>Key</Text>
              <CopyText
                content={rotated.key}
                label="API key"
                w={{ base: 200, lg: 250 }}
              />
            </Group>

            <Group justify="space-between" wrap="nowrap">
              <Text>Secret</Text>
              <CopyText
                content={rotated.secret}
                label="API secret"
                w={{ base: 200, lg: 250 }}
              />
            </Group>

            <Group justify="end">
              <Button
                leftSection={<ICONS.Clear />}
                onClick={() => setRotated(undefined)}
              >
                Close
              </Button>
            </Group>
          </Stack>
        )}
      </Modal>
    </Section>
  );
}
