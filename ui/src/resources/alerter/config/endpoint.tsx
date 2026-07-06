import { MonacoEditor, ConfigInput, ConfigItem } from "mogh_ui";
import { Select } from "@mantine/core";
import { Types } from "komodo_client";

const ENDPOINT_TYPES: {
  value: Types.AlerterEndpoint["type"];
  label: string;
}[] = [
  { value: "Custom", label: "Custom" },
  { value: "Discord", label: "Discord" },
  { value: "Slack", label: "Slack" },
  { value: "Ntfy", label: "Ntfy" },
  { value: "Pushover", label: "Pushover" },
  { value: "Mqtt", label: "MQTT" },
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
            set(defaultEndpoint(type as Types.AlerterEndpoint["type"]))
          }
          disabled={disabled}
          data={ENDPOINT_TYPES}
          w={{ base: "85%", lg: 400 }}
        />
        {endpoint.type !== "Mqtt" && (
          <MonacoEditor
            value={endpoint.params.url}
            language={undefined}
            onValueChange={(url) =>
              set({ ...endpoint, params: { ...endpoint.params, url } })
            }
            readOnly={disabled}
          />
        )}
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
      {endpoint.type === "Mqtt" && (
        <>
          <ConfigInput
            label="Broker URL"
            description="Required. MQTT broker URL, for example mqtt://localhost:1883."
            placeholder="mqtt://localhost:1883"
            value={endpoint.params.broker_url}
            onValueChange={(broker_url) =>
              set({
                ...endpoint,
                params: { ...endpoint.params, broker_url },
              })
            }
            disabled={disabled}
          />
          <ConfigInput
            label="Topic"
            description="Required. Topic to publish the alert JSON to."
            placeholder="komodo/events"
            value={endpoint.params.topic}
            onValueChange={(topic) =>
              set({
                ...endpoint,
                params: { ...endpoint.params, topic },
              })
            }
            disabled={disabled}
          />
          <ConfigInput
            label="Username"
            description="Optional broker username."
            placeholder="username"
            value={endpoint.params.username ?? ""}
            onValueChange={(username) =>
              set({
                ...endpoint,
                params: { ...endpoint.params, username: username || undefined },
              })
            }
            disabled={disabled}
          />
          <ConfigInput
            label="Password"
            description="Optional broker password."
            placeholder="password"
            value={endpoint.params.password ?? ""}
            onValueChange={(password) =>
              set({
                ...endpoint,
                params: { ...endpoint.params, password: password || undefined },
              })
            }
            inputProps={{ type: "password" }}
            disabled={disabled}
          />
          <ConfigInput
            label="Client ID"
            description="Optional MQTT client identifier. If empty, Komodo generates one."
            placeholder="komodo-core"
            value={endpoint.params.client_id ?? ""}
            onValueChange={(client_id) =>
              set({
                ...endpoint,
                params: {
                  ...endpoint.params,
                  client_id: client_id || undefined,
                },
              })
            }
            disabled={disabled}
          />
          <ConfigItem
            label="QoS"
            description="Optional quality of service level."
          >
            <Select
              value={String(endpoint.params.qos ?? 0)}
              data={[
                { value: "0", label: "0" },
                { value: "1", label: "1" },
                { value: "2", label: "2" },
              ]}
              onChange={(qos) =>
                set({
                  ...endpoint,
                  params: {
                    ...endpoint.params,
                    qos: Number(qos ?? "0"),
                  },
                })
              }
              disabled={disabled}
              w={{ base: "85%", lg: 400 }}
            />
          </ConfigItem>
          <ConfigItem
            label="Retain"
            description="Optional retain flag for published messages."
          >
            <Select
              value={(endpoint.params.retain ?? false) ? "true" : "false"}
              data={[
                { value: "false", label: "false" },
                { value: "true", label: "true" },
              ]}
              onChange={(retain) =>
                set({
                  ...endpoint,
                  params: {
                    ...endpoint.params,
                    retain: retain === "true",
                  },
                })
              }
              disabled={disabled}
              w={{ base: "85%", lg: 400 }}
            />
          </ConfigItem>
        </>
      )}
    </>
  );
}

function defaultEndpoint(
  type: Types.AlerterEndpoint["type"],
): Types.AlerterEndpoint {
  return type === "Mqtt"
    ? {
        type,
        params: {
          broker_url: "mqtt://localhost:1883",
          topic: "komodo/events",
          username: undefined,
          password: undefined,
          client_id: undefined,
          qos: 0,
          retain: false,
        },
      }
    : {
        type,
        params: {
          url: defaultUrl(type),
        },
      };
}

function defaultUrl(
  type: Exclude<Types.AlerterEndpoint["type"], "Mqtt">,
) {
  return type === "Custom"
    ? "http://localhost:7000"
    : type === "Slack"
      ? "https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXXXXXX"
      : type === "Discord"
        ? "https://discord.com/api/webhooks/XXXXXXXXXXXX/XXXX-XXXXXXXXXX"
        : type === "Ntfy"
          ? "https://ntfy.sh/komodo"
          : "https://api.pushover.net/1/messages.json?token=XXXXXXXXXXXXX&user=XXXXXXXXXXXXX";
}
