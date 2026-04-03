import { MonacoEditor, ConfigInput, ConfigItem } from "mogh_ui";
import { Select } from "@mantine/core";
import { Types } from "komodo_client";

const ENDPOINT_TYPES: Types.AlerterEndpoint["type"][] = [
  "Custom",
  "Discord",
  "Slack",
  "Ntfy",
  "Pushover",
] as const;

export default function AlerterConfigEndpoint({
  endpoint,
  set,
  disabled,
}: {
  endpoint: Types.AlerterEndpoint;
  set: (endpoint: Types.AlerterEndpoint) => void;
  disabled: boolean;
}) {
  return (
    <>
      <ConfigItem
        label="Endpoint"
        description="Configure the endpoint to send the alert to."
      >
        <Select
          value={endpoint.type}
          onChange={(type) =>
            type &&
            set({
              type: type as Types.AlerterEndpoint["type"],
              params: {
                url: defaultUrl(type as Types.AlerterEndpoint["type"]),
              },
            })
          }
          disabled={disabled}
          data={ENDPOINT_TYPES}
          w={{ base: "85%", lg: 400 }}
        />
        <MonacoEditor
          value={endpoint.params.url}
          language={undefined}
          onValueChange={(url) =>
            set({ ...endpoint, params: { ...endpoint.params, url } })
          }
          readOnly={disabled}
        />
      </ConfigItem>
      {endpoint.type === "Ntfy" && (
        <ConfigInput
          label="Email"
          description="Request Ntfy to send an email to this address. SMTP must be configured on the Ntfy instance. Only one email address per alerter is supported."
          placeholder="john@example.com"
          value={endpoint.params.email}
          onValueChange={(email) =>
            set({
              ...endpoint,
              params: { ...endpoint.params, email },
            })
          }
          disabled={disabled}
          email
        />
      )}
    </>
  );
}

function defaultUrl(type: Types.AlerterEndpoint["type"]) {
  return type === "Custom"
    ? "http://localhost:7000"
    : type === "Slack"
      ? "https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXXXXXX"
      : type === "Discord"
        ? "https://discord.com/api/webhooks/XXXXXXXXXXXX/XXXX-XXXXXXXXXX"
        : type === "Ntfy"
          ? "https://ntfy.sh/komodo"
          : type === "Pushover"
            ? "https://api.pushover.net/1/messages.json?token=XXXXXXXXXXXXX&user=XXXXXXXXXXXXX"
            : "";
}
