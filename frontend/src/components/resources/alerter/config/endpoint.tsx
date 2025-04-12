import { ConfigItem } from "@components/config/util";
import { MonacoEditor } from "@components/monaco";
import { Types } from "komodo_client";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";

const ENDPOINT_TYPES: Types.AlerterEndpoint["type"][] = [
  "Custom",
  "Discord",
  "Slack",
  "Ntfy",
];

export const EndpointConfig = ({
  endpoint,
  set,
  disabled,
}: {
  endpoint: Types.AlerterEndpoint;
  set: (endpoint: Types.AlerterEndpoint) => void;
  disabled: boolean;
}) => {
  return (
    <ConfigItem
      label="Endpoint"
      description="Configure the endpoint to send the alert to."
      boldLabel
    >
      <Select
        value={endpoint.type}
        onValueChange={(type: Types.AlerterEndpoint["type"]) => {
          set({ type, params: { url: default_url(type) } });
        }}
        disabled={disabled}
      >
        <SelectTrigger className="w-[150px]" disabled={disabled}>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {ENDPOINT_TYPES.map((endpoint) => (
            <SelectItem key={endpoint} value={endpoint}>
              {endpoint}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <MonacoEditor
        value={endpoint.params.url}
        language={undefined}
        onValueChange={(url) =>
          set({ ...endpoint, params: { ...endpoint.params, url } })
        }
        readOnly={disabled}
      />
    </ConfigItem>
  );
};

const default_url = (type: Types.AlerterEndpoint["type"]) => {
  return type === "Custom"
    ? "http://localhost:7000"
    : type === "Slack"
    ? "https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXXXXXX"
    : type === "Discord"
    ? "https://discord.com/api/webhooks/XXXXXXXXXXXX/XXXX-XXXXXXXXXX"
    : type === "Ntfy"
    ? "https://ntfy.sh/komodo"
    : "";
};
