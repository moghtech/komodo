import { useInvalidate, useWrite } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import {
  Button,
  Group,
  Modal,
  Stack,
  Switch,
  Text,
  TextInput,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { notifications } from "@mantine/notifications";
import { CircleCheckBig } from "lucide-react";
import { ChangeEvent, useState } from "react";

export default function NewProviderAccount({
  type,
}: {
  type: "GitProvider" | "DockerRegistry";
}) {
  const [opened, { open, close }] = useDisclosure();
  const [domain, setDomain] = useState("");
  const [https, setHttps] = useState(true);
  const [username, setUsername] = useState("");
  const [token, setToken] = useState("");
  const invalidate = useInvalidate();
  const { mutate: create, isPending } = useWrite(`Create${type}Account`, {
    onSuccess: () => {
      invalidate([`List${type}Accounts`]);
      notifications.show({ message: "Account created" });
      close();
    },
  });
  const submit = () => create({ account: { domain, https, username, token } });

  const form: Array<
    | undefined
    | [string, string, (e: ChangeEvent<HTMLInputElement>) => void, false]
    | [string, boolean, (checked: boolean) => void, true]
  > = [
    [
      "Domain",
      domain,
      (e: ChangeEvent<HTMLInputElement>) => setDomain(e.target.value),
      false,
    ],
    type === "GitProvider"
      ? ["Use HTTPS", https, (https: boolean) => setHttps(https), true]
      : undefined,
    [
      "Username",
      username,
      (e: ChangeEvent<HTMLInputElement>) => setUsername(e.target.value),
      false,
    ],
    [
      "Token",
      token,
      (e: ChangeEvent<HTMLInputElement>) => setToken(e.target.value),
      false,
    ],
  ];
  const accountType =
    type === "DockerRegistry" ? "Registry Account" : "Git Account";

  return (
    <>
      <Modal
        opened={opened}
        onClose={close}
        title={<Text size="lg">Create {accountType}</Text>}
        size="lg"
      >
        <Stack>
          {form.map((item) => {
            if (!item) return;

            const [title, value, onChange, bool] = item;

            if (bool) {
              return (
                <Group key={title} justify="space-between">
                  {title}
                  <Switch
                    checked={value}
                    onChange={(e) => onChange(e.target.checked)}
                  />
                </Group>
              );
            }

            return (
              <Group key={title} justify="space-between">
                {title}
                <TextInput
                  placeholder={`Input ${title.toLowerCase()}`}
                  value={value}
                  onChange={onChange}
                  w={{ base: 200, lg: 300 }}
                />
              </Group>
            );
          })}

          <Group justify="end">
            <Button
              leftSection={<CircleCheckBig size="1rem" />}
              onClick={submit}
              loading={isPending}
            >
              Create
            </Button>
          </Group>
        </Stack>
      </Modal>

      <Button leftSection={<ICONS.Create size="1rem" />} onClick={open}>
        New Account
      </Button>
    </>
  );
}
