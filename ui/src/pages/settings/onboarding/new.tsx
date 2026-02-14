import { useInvalidate, useWrite } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import CopyButton from "@/ui/copy-button";
import {
  Button,
  Group,
  Modal,
  PasswordInput,
  Select,
  Stack,
  Text,
  TextInput,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { notifications } from "@mantine/notifications";
import { useState } from "react";

type ExpiresOptions = "1 day" | "7 days" | "30 days" | "never";
const ONE_DAY_MS = 1000 * 60 * 60 * 24;

export default function NewOnboardingKey() {
  const [opened, { open, close: _close }] = useDisclosure();
  const [name, setName] = useState("");
  const [privateKey, setPrivateKey] = useState("");
  const [expires, setExpires] = useState<ExpiresOptions>("1 day");

  const [submitted, setSubmitted] = useState<{ private_key: string }>();
  const invalidate = useInvalidate();
  const { mutate, isPending } = useWrite("CreateOnboardingKey", {
    onSuccess: ({ private_key }) => {
      notifications.show({ message: "Onboarding Key Created" });
      invalidate(["ListOnboardingKeys"]);
      setSubmitted({ private_key });
    },
  });
  const now = Date.now();
  const expiresOptions: Record<ExpiresOptions, number> = {
    "1 day": now + ONE_DAY_MS,
    "7 days": now + ONE_DAY_MS * 7,
    "30 days": now + ONE_DAY_MS * 90,
    never: 0,
  };
  const submit = () =>
    mutate({
      name,
      expires: expiresOptions[expires],
      private_key: privateKey || undefined,
    });

  const close = () => {
    setName("");
    setPrivateKey("");
    setExpires("1 day");
    setSubmitted(undefined);
    _close();
  };

  return (
    <>
      <Modal
        opened={opened}
        onClose={close}
        title={
          <Text size="lg">
            {!submitted && "Create Onboarding Key"}
            {submitted && "Onboarding Key Created"}
          </Text>
        }
        size="lg"
      >
        <Stack>
          {!submitted && (
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
                <Text>Pre-Existing Key</Text>
                <PasswordInput
                  value={privateKey}
                  onChange={(e) => setPrivateKey(e.target.value)}
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
                  onClick={submit}
                  leftSection={<ICONS.Check size="1rem" />}
                  loading={isPending}
                >
                  Submit
                </Button>
              </Group>
            </>
          )}

          {submitted && (
            <>
              <Text>
                Use as the <b>PERIPHERY_ONBOARDING_KEY</b>
              </Text>

              <Group>
                <Text
                  ff="monospace"
                  title={submitted.private_key}
                  className="text-ellipsis"
                  bg="accent.3"
                  bd="1px solid var(--mantine-color-accent-border-4)"
                  bdrs="md"
                  px="md"
                  py="0.4rem"
                  w="calc(100% - 60px)"
                >
                  {submitted.private_key}
                </Text>
                <CopyButton content={submitted.private_key} />
              </Group>

              <Group justify="end" onClick={close}>
                <Button leftSection={<ICONS.Clear />}>Close</Button>
              </Group>
            </>
          )}
        </Stack>
      </Modal>

      <Button leftSection={<ICONS.Create size="1rem" />} onClick={open}>
        New Onboarding Key
      </Button>
    </>
  );
}
