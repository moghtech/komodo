import { useRead, WebhookIntegration } from "@/lib/hooks";
import { ConfigItem, ConfigItemProps } from "@/ui/config/item";
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
  const url = baseUrl + "/listener/" + integration.toLowerCase() + path;
  return (
    <ConfigItem label="Webhook Url" {...itemProps}>
      <CopyText content={url} />
    </ConfigItem>
  );
}
