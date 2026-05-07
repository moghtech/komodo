import { MonacoEditor } from "@/components/monaco";
import { recordToText, textToRecord } from "@/lib/utils";
import { ConfigInput, ConfigItem } from "@/ui/config/item";
import { Select, Text } from "@mantine/core";
import { Types } from "komodo_client";
import { useEffect, useState } from "react";

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
  onValidationChange,
}: {
  endpoint: Types.AlerterEndpoint;
  set: (endpoint: Types.AlerterEndpoint) => void;
  disabled: boolean;
  onValidationChange: (error: string | undefined) => void;
}) {
  const headersValue =
    endpoint.type === "Custom"
      ? recordToText(endpoint.params.headers)
      : "";
  const [headersText, setHeadersText] = useState(headersValue);
  const [headersError, setHeadersError] = useState<string | undefined>();

  useEffect(() => {
    setHeadersText(headersValue);
    setHeadersError(undefined);
    onValidationChange(undefined);
  }, [endpoint, headersValue, onValidationChange]);

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
                ...(type === "Custom" ? { headers: {} } : {}),
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
      {endpoint.type === "Custom" && (
        <ConfigItem
          label="Headers"
          description="Additional request headers to include when posting the JSON alert payload. Use one `Header: value` pair per line."
        >
          <MonacoEditor
            value={headersText}
            language="key_value"
            onValueChange={(headersText) => {
              setHeadersText(headersText);
              try {
                const headers = textToRecord(headersText);
                setHeadersError(undefined);
                onValidationChange(undefined);
                set({
                  ...endpoint,
                  params: { ...endpoint.params, headers },
                });
              } catch (error) {
                const message =
                  error instanceof Error
                    ? error.message
                    : "Invalid header format.";
                setHeadersError(message);
                onValidationChange(message);
              }
            }}
            readOnly={disabled}
          />
          {headersError && (
            <Text c="red" size="sm" mt="xs">
              {headersError}
            </Text>
          )}
        </ConfigItem>
      )}
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
