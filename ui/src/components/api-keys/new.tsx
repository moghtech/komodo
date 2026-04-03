import { useInvalidate, useManageAuth, useWrite } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import CopyText from "@/ui/copy-text";
import {
  Button,
  Group,
  Modal,
  Select,
  Stack,
  Text,
  TextInput,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { Types } from "komodo_client";
import { useState } from "react";

const ONE_DAY_MS = 1000 * 60 * 60 * 24;
type ExpiresOptions = "90 days" | "180 days" | "1 year" | "Never";

export interface NewApiKeyProps {
  /** For service user api keys */
  userId?: string;
}

export default function NewApiKey({ userId }: { userId?: string }) {
  const [opened, { open, close: _close }] = useDisclosure();
  const [name, setName] = useState("");
  const [expires, setExpires] = useState<ExpiresOptions>("90 days");
  const [created, setCreated] = useState<Types.CreateApiKeyResponse>();
  const invalidate = useInvalidate();
  const { mutate: regularCreate, isPending: regularPending } = useManageAuth(
    "CreateApiKey",
    {
      onSuccess: (res) => {
        invalidate(["ListApiKeys"]);
        setCreated(res);
      },
    },
  );
  const { mutate: serviceCreate, isPending: servicePending } = useWrite(
    "CreateApiKeyForServiceUser",
    {
      onSuccess: (res) => {
        invalidate(["ListApiKeysForServiceUser"]);
        setCreated(res);
      },
    },
  );
  const now = Date.now();
  const expiresOptions: Record<ExpiresOptions, number> = {
    "90 days": now + ONE_DAY_MS * 90,
    "180 days": now + ONE_DAY_MS * 180,
    "1 year": now + ONE_DAY_MS * 365,
    Never: 0,
  };
  const create = () => {
    const data = {
      name,
      expires: expiresOptions[expires],
    };
    if (userId) {
      serviceCreate({
        user_id: userId,
        ...data,
      });
    } else {
      regularCreate(data);
    }
  };

  const close = () => {
    setName("");
    setExpires("90 days");
    setCreated(undefined);
    _close();
  };

  return (
    <>
      <Modal
        opened={opened}
        onClose={close}
        title={
          <Text size="lg">
            {!created && "Create API Key" + (userId ? " for Service User" : "")}
            {created && "API Key Created"}
          </Text>
        }
      >
        <Stack>
          {!created && (
            <>
              <Group justify="space-between">
                <Text>Name</Text>
                <TextInput
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="Optional"
                  w={{ base: 200, lg: 300 }}
                />
              </Group>

              <Group justify="space-between">
                <Text>Expiry</Text>
                <Select
                  value={expires}
                  onChange={(expires) => expires && setExpires(expires as any)}
                  data={Object.keys(expiresOptions)}
                  w={{ base: 200, lg: 300 }}
                />
              </Group>

              <Group justify="end">
                <Button
                  variant="secondary"
                  className="gap-4"
                  onClick={create}
                  leftSection={<ICONS.Check size="1rem" />}
                  loading={userId ? servicePending : regularPending}
                >
                  Create
                </Button>
              </Group>
            </>
          )}

          {created && (
            <>
              <Text>
                Copy the API key and secret.{" "}
                <b>The secret will not be shown again.</b>
              </Text>

              <Group justify="space-between" wrap="nowrap">
                <Text>Key</Text>
                <CopyText content={created.key} label="API key" w={{ base: 200, lg: 250 }} />
              </Group>

              <Group justify="space-between" wrap="nowrap">
                <Text>Secret</Text>
                <CopyText content={created.secret} label="API secret" w={{ base: 200, lg: 250 }} />
              </Group>

              <Group justify="end" onClick={close}>
                <Button leftSection={<ICONS.Clear />}>Close</Button>
              </Group>
            </>
          )}
        </Stack>
      </Modal>

      <Button leftSection={<ICONS.Create size="1rem" />} onClick={open}>
        New API Key
      </Button>
    </>
  );
}
