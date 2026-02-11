import { useExecute, usePermissions, useRead } from "@/lib/hooks";
import { ReactNode } from "react";
import { useFullResourceSync } from ".";
import { useResourceSyncTabsView } from "./hooks";
import Section from "@/ui/section";
import { Code, Group, Stack, Text } from "@mantine/core";
import { sanitizeOnlySpan } from "@/lib/utils";
import { Types } from "komodo_client";
import DividedChildren from "@/ui/divided-children";
import ResourceLink from "@/resources/link";
import { UsableResource } from "@/resources";
import ConfirmButton from "@/ui/confirm-button";
import { SquarePlay } from "lucide-react";
import { MonacoDiffEditor, MonacoEditor } from "@/components/monaco";

export default function ResourceSyncPending({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const syncing = useRead("GetResourceSyncActionState", { sync: id }).data
    ?.syncing;
  const sync = useFullResourceSync(id);
  const { view } = useResourceSyncTabsView(sync);
  const { canExecute } = usePermissions({ type: "ResourceSync", id });
  const { mutate: runSync, isPending } = useExecute("RunSync");
  const loading = isPending || syncing;

  return (
    <Section titleOther={titleOther} mih={500}>
      {/* INFO ON EXECUTE / COMMIT MODE */}
      <Group>
        <Text c="dimmed">{view} Mode:</Text>
        {view === "Execute" && (
          <Text>
            Update resources in the
            <b>UI</b>
            to match the
            <b>file changes.</b>
          </Text>
        )}
        {view === "Commit" && (
          <Text>
            Update resources in the
            <b>file</b>
            to match the
            <b>UI changes.</b>
          </Text>
        )}
      </Group>

      {/* PENDING ERROR */}
      {!!sync?.info?.pending_error && (
        <Stack
          bd="1px solid var(--mantine-color-accent-border-0)"
          bdrs="md"
          p="xl"
        >
          <Text c="red" ff="monospace">
            Error
          </Text>
          <Code
            component="pre"
            fz="sm"
            mah={500}
            dangerouslySetInnerHTML={{
              __html: sanitizeOnlySpan(sync.info.pending_error),
            }}
            style={{ overflowY: "auto" }}
          />
        </Stack>
      )}

      {/* PENDING DEPLOY */}
      {view === "Execute" && !!sync?.info?.pending_deploy?.to_deploy && (
        <Stack
          bd="1px solid var(--mantine-color-accent-border-0)"
          bdrs="md"
          p="xl"
        >
          <Text c="yellow" ff="monospace">
            Deploy {sync.info.pending_deploy.to_deploy} Resource
            {sync.info.pending_deploy.to_deploy > 1 ? "s" : ""}
          </Text>
          <Code
            component="pre"
            fz="sm"
            mah={500}
            dangerouslySetInnerHTML={{
              __html: sanitizeOnlySpan(sync.info.pending_deploy.log),
            }}
            style={{ overflowY: "auto" }}
          />
        </Stack>
      )}

      {/* PENDING RESOURCE UPDATES */}
      {sync?.info?.resource_updates?.map((update) => {
        return (
          <Stack
            key={update.target.type + update.target.id}
            bd="1px solid var(--mantine-color-accent-border-0)"
            bdrs="md"
            p="xl"
          >
            {/* HEADER */}
            <Group justify="space-between">
              <DividedChildren>
                <Text ff="monospace">
                  {view === "Commit"
                    ? reversePendingType(update.data.type)
                    : update.data.type}{" "}
                  {update.target.type}
                </Text>
                {update.data.type === "Create" ? (
                  <Text>{update.data.data.name}</Text>
                ) : (
                  <ResourceLink
                    type={update.target.type as UsableResource}
                    id={update.target.id}
                  />
                )}
              </DividedChildren>
              {canExecute && view === "Execute" && (
                <ConfirmButton
                  icon={<SquarePlay className="w-4 h-4" />}
                  onClick={() =>
                    runSync({
                      sync: id,
                      resource_type: update.target.type,
                      resources: [
                        update.data.type === "Create"
                          ? update.data.data.name!
                          : update.target.id,
                      ],
                    })
                  }
                  loading={loading}
                >
                  Execute Change
                </ConfirmButton>
              )}
            </Group>

            {/* CONTENT */}
            {update.data.type === "Create" && (
              <MonacoEditor
                value={update.data.data.proposed}
                language="fancy_toml"
                readOnly
              />
            )}
            {update.data.type === "Update" && (
              <>
                {view === "Execute" && (
                  <MonacoDiffEditor
                    original={update.data.data.current}
                    modified={update.data.data.proposed}
                    language="fancy_toml"
                    readOnly
                  />
                )}
                {view === "Commit" && (
                  <MonacoDiffEditor
                    original={update.data.data.proposed}
                    modified={update.data.data.current}
                    language="fancy_toml"
                    readOnly
                  />
                )}
              </>
            )}
            {update.data.type === "Delete" && (
              <MonacoEditor
                value={update.data.data.current}
                language="fancy_toml"
                readOnly
              />
            )}
          </Stack>
        );
      })}

      {/* PENDING VARIABLE UPDATES */}
      {sync?.info?.variable_updates?.map((data, i) => {
        return (
          <Stack
            key={i}
            bd="1px solid var(--mantine-color-accent-border-0)"
            bdrs="md"
            p="xl"
          >
            <Text ff="monospace">
              {view === "Commit" ? reversePendingType(data.type) : data.type}{" "}
              Variable
            </Text>
            {data.type === "Create" && (
              <MonacoEditor
                value={data.data.proposed}
                language="fancy_toml"
                readOnly
              />
            )}
            {data.type === "Update" && (
              <>
                {view === "Execute" && (
                  <MonacoDiffEditor
                    original={data.data.current}
                    modified={data.data.proposed}
                    language="fancy_toml"
                    readOnly
                  />
                )}
                {view === "Commit" && (
                  <MonacoDiffEditor
                    original={data.data.proposed}
                    modified={data.data.current}
                    language="fancy_toml"
                    readOnly
                  />
                )}
              </>
            )}
            {data.type === "Delete" && (
              <MonacoEditor
                value={data.data.current}
                language="fancy_toml"
                readOnly
              />
            )}
          </Stack>
        );
      })}

      {/* PENDING USER GROUP UPDATES */}
      {sync?.info?.user_group_updates?.map((data, i) => {
        return (
          <Stack
            key={i}
            bd="1px solid var(--mantine-color-accent-border-0)"
            bdrs="md"
            p="xl"
          >
            <Text ff="monospace">
              {view === "Commit" ? reversePendingType(data.type) : data.type}{" "}
              User Group
            </Text>
            {data.type === "Create" && (
              <MonacoEditor
                value={data.data.proposed}
                language="fancy_toml"
                readOnly
              />
            )}
            {data.type === "Update" && (
              <>
                {view === "Execute" && (
                  <MonacoDiffEditor
                    original={data.data.current}
                    modified={data.data.proposed}
                    language="fancy_toml"
                    readOnly
                  />
                )}
                {view === "Commit" && (
                  <MonacoDiffEditor
                    original={data.data.proposed}
                    modified={data.data.current}
                    language="fancy_toml"
                    readOnly
                  />
                )}
              </>
            )}
            {data.type === "Delete" && (
              <MonacoEditor
                value={data.data.current}
                language="fancy_toml"
                readOnly
              />
            )}
          </Stack>
        );
      })}
    </Section>
  );
}

function reversePendingType(type: Types.ResourceDiff["data"]["type"]) {
  switch (type) {
    case "Create":
      return "Remove";
    case "Update":
      return "Update";
    case "Delete":
      return "Add";
  }
}
