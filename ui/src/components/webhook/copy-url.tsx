import { useRead, WebhookIntegration } from "@/lib/hooks";
import { ConfigItem, ConfigItemProps } from "@/ui/config/item";
import { Text } from "@mantine/core";
import CopyText from "@/ui/copy-text";

export interface CopyWebhookUrlProps extends Omit<ConfigItemProps, "children"> {
  integration: WebhookIntegration;
  path: string;
}

export default function CopyWebhookUrl({
  integration,
  path,
  ...itemProps
}: CopyWebhookUrlProps) {
  const baseUrl = useRead("GetCoreInfo", {}).data?.webhook_base_url;
  let url = baseUrl + "/listener/" + integration.toLowerCase() + path;
  
  let description = undefined;
  if (integration === "Query") {
    url += "?secret=YOUR_SECRET_HERE";
    description = (
      <Text size="sm" c="dimmed">
        Requires <code>?secret=...</code> to match the configured webhook secret. Optionally supports <code>&branch=...</code> to trigger on a specific branch (defaults to main).
      </Text>
    );
  }

  return (
    <ConfigItem label="Webhook URL" description={description} {...itemProps}>
      <CopyText content={url} />
    </ConfigItem>
  );
}
