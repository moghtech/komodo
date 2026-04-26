import {
  useWebhookIdOrName,
  useWebhookIntegrations,
  WebhookIdOrName,
  WebhookIntegration,
} from "@/lib/hooks";
import { ConfigItem, ConfigItemProps } from "@/ui/config/item";
import { Select, Stack } from "@mantine/core";
import { ReactNode } from "react";

export interface WebhookBuilderProps extends Omit<ConfigItemProps, "children"> {
  gitProvider: string;
  extra?: ReactNode;
}

export default function WebhookBuilder({
  gitProvider,
  extra,
  ...itemProps
}: WebhookBuilderProps) {
  const { setIntegration, getIntegration } = useWebhookIntegrations();
  const [idOrName, setIdOrName] = useWebhookIdOrName();
  return (
    <ConfigItem {...itemProps}>
      <Stack gap="xs">
        <Select
          label="Auth style?"
          value={getIntegration(gitProvider)}
          onChange={(integration) =>
            integration &&
            setIntegration(gitProvider, integration as WebhookIntegration)
          }
          data={["Github", "Gitlab", "Query"]}
          w={{ base: "100%", sm: 200 }}
        />
        <Select
          label="Resource Id or Name?"
          value={idOrName}
          onChange={(value) => value && setIdOrName(value as WebhookIdOrName)}
          data={["Id", "Name"]}
          w={{ base: "100%", sm: 200 }}
        />
        {extra}
      </Stack>
    </ConfigItem>
  );
}
