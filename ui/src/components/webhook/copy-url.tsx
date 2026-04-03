import { useRead, WebhookIntegration } from "@/lib/hooks";
import { ConfigItem, ConfigItemProps } from "mogh_ui";
import { CopyText } from "mogh_ui";

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
    <ConfigItem label="Webhook URL" {...itemProps}>
      <CopyText content={url} />
    </ConfigItem>
  );
}
