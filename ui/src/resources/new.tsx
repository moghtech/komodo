import { ReactNode, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useRead, useWrite } from "@/lib/hooks";
import { UsableResource } from ".";
import { notifications } from "@mantine/notifications";
import { usableResourcePath } from "@/lib/utils";
import { CreateModal } from "mogh_ui";
import { Divider, Stack, Text, TextInput } from "@mantine/core";
import ResourceSelector from "./selector";
import { Types } from "komodo_client";
import { Copy } from "lucide-react";

export interface NewResourceProps<Config> {
  config?: () => Partial<Config>;
  copyId?: string;
  type: UsableResource;
  readableType?: string;
  name?: string;
  extraInputs?: ReactNode;
  showTemplateSelector?: boolean;
}

export default function NewResource<Config>({
  config,
  copyId = "",
  type,
  readableType,
  name: _name = "",
  extraInputs,
  showTemplateSelector: _showTemplateSelector = true,
}: NewResourceProps<Config>) {
  const nav = useNavigate();
  const showTemplateSelector =
    (useRead(`List${type}s`, {}).data?.filter((r) => r.template).length ?? 0) >
    0;

  const { mutateAsync: create, isPending: createPending } = useWrite(
    `Create${type}`,
  );

  const { mutateAsync: copy, isPending: copyPending } = useWrite(`Copy${type}`);

  const [templateId, setTemplateId] = useState(copyId);
  const [name, setName] = useState(_name);

  const placeholderType =
    type === "ResourceSync" ? "resource-sync" : type.toLowerCase();

  const onConfirm = async () => {
    if (!name.trim()) {
      notifications.show({ message: "Name cannot be empty", color: "red" });
      return false;
    }
    try {
      const result = templateId
        ? await copy({ name: name.trim(), id: templateId })
        : await create({ name: name.trim(), config: config?.() ?? {} });
      const resourceId = result._id?.$oid;
      if (resourceId) {
        nav(`/${usableResourcePath(type)}/${resourceId}`);
      }
      return true;
    } catch (error) {
      notifications.show({ message: "Unexpected error.", color: "red" });
      return false;
    }
  };

  return (
    <CreateModal
      entityType={readableType ?? type}
      onConfirm={onConfirm}
      disabled={false}
      loading={createPending || copyPending}
      openShiftKeyListener="N"
      leftSection={copyId && <Copy size="1rem" />}
      configSection={() => (
        <Stack>
          <TextInput
            autoFocus
            placeholder={`${placeholderType}-name`}
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => {
              if (!name) {
                return;
              }
              if (e.key === "Enter") {
                onConfirm();
              }
            }}
            error={!name.trim() && "Enter name"}
          />

          {!templateId && extraInputs}

          {(!!copyId || (_showTemplateSelector && showTemplateSelector)) && (
            <>
              {!templateId && extraInputs && <Divider />}
              <ResourceSelector
                type={type}
                selected={templateId}
                onSelect={setTemplateId}
                templates={
                  copyId ? undefined : Types.TemplatesQueryBehavior.Only
                }
                wrapperProps={
                  copyId
                    ? {
                        label: (
                          <Text mb="4" size="sm" c="dimmed">
                            Copy Source
                          </Text>
                        ),
                      }
                    : undefined
                }
                placeholder="Select Template"
                targetProps={{ w: "100%", maw: "100%" }}
                width="target"
                position="bottom"
                disabled={!!copyId}
                clearable
              />
            </>
          )}
        </Stack>
      )}
    >
      {copyId ? "Copy" : null}
    </CreateModal>
  );
}
