import { useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { Button, Group, Stack, Text } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { MinusCircle } from "lucide-react";
import { usableResourcePath } from "@/lib/utils";
import { TemplateMarker } from "@/components/template-marker";
import { ICONS } from "@/lib/icons";
import { useInvalidate, usePermissions, useRead, useWrite } from "@/lib/hooks";
import { ResourceComponents, UsableResource, UsableResourceTarget } from ".";
import Tags from "@/components/tags";
import TextUpdateModal from "@/ui/text-update-modal";

export const ResourceNameSimple = ({
  type,
  id,
}: {
  type: UsableResource;
  id: string;
}) => {
  const Components = ResourceComponents[type];
  const name = Components.useListItem(id)?.name ?? "unknown";
  return <Text>{name}</Text>;
};

export const ResourceLink = ({
  type,
  id,
  onClick,
}: {
  type: UsableResource;
  id: string;
  onClick?: () => void;
}) => {
  const Components = ResourceComponents[type];
  const resource = Components.useListItem(id);
  return (
    <Group
      renderRoot={(props) => (
        <Link to={`/${usableResourcePath(type)}/${id}`} {...props} />
      )}
      onClick={(e) => {
        e.stopPropagation();
        onClick?.();
      }}
      style={{ textDecoration: "underline" }}
      c="inherit"
      wrap="nowrap"
    >
      <Components.Icon id={id} />
      <Text style={{ textWrap: "nowrap" }}>{resource?.name ?? "Unknown"}</Text>
      {resource?.template && <TemplateMarker type={type} />}
    </Group>
  );
};

export const ResourceNotFound = ({
  type,
}: {
  type: UsableResource | undefined;
}) => {
  const nav = useNavigate();
  const Components = type && ResourceComponents[type];
  return (
    <Stack gap="md">
      {type && (
        <Group mb="xl">
          <Button
            variant="default"
            leftSection={<ICONS.Back size="1rem" />}
            onClick={() => nav("/" + usableResourcePath(type))}
          >
            Back
          </Button>
        </Group>
      )}
      <Group gap="lg">
        <div className="mt-1">
          {Components ? (
            <Components.Icon size="2em" />
          ) : (
            <ICONS.NotFound size="2rem" />
          )}
        </div>
        <Text fz="h1" ff="monospace">
          {type} {type && " - "} 404 Not Found
        </Text>
      </Group>
    </Stack>
  );
};

export const ResourceTags = ({
  target,
  click_to_delete,
  disabled,
}: {
  target: UsableResourceTarget;
  click_to_delete?: boolean;
  disabled?: boolean;
}) => {
  const inv = useInvalidate();
  const { type, id } = target;
  const resource = useRead(`List${type}s`, {}).data?.find((d) => d.id === id);
  const { mutate } = useWrite("UpdateResourceMeta", {
    onSuccess: () => {
      inv([`List${type}s`]);
      notifications.show({ message: "Removed tag" });
    },
  });
  return (
    <Tags
      tag_ids={resource?.tags}
      onBadgeClick={(tag_id) => {
        if (!click_to_delete) return;
        if (disabled) return;
        mutate({
          target,
          tags: resource!.tags.filter((tag) => tag !== tag_id),
        });
      }}
      icon={!disabled && click_to_delete && <MinusCircle size="1rem" />}
    />
  );
};

export const ResourceDescription = ({
  type,
  id,
}: {
  type: UsableResource;
  id: string;
}) => {
  const { canWrite } = usePermissions({ type, id });
  const inv = useInvalidate();
  const key = type === "ResourceSync" ? "sync" : type.toLowerCase();

  const [open, setOpen] = useState(false);

  const resource = useRead(`Get${type}`, {
    [key]: id,
  } as any).data;

  const { mutate: updateDescription } = useWrite("UpdateResourceMeta", {
    onSuccess: () => {
      inv([`Get${type}`]);
      notifications.show({
        message: `Updated description on ${type} '${resource?.name}'`,
      });
    },
  });

  return (
    <>
      <TextUpdateModal
        title="Update Description"
        placeholder="Set Description"
        value={resource?.description}
        onUpdate={(description) =>
          updateDescription({ target: { type, id }, description })
        }
        disabled={!canWrite}
        open={open}
        setOpen={setOpen}
      />

      <Button
        variant="outline"
        c="dimmed"
        p="md"
        bd="1px solid var(--mantine-color-accent-border-0)"
        bdrs="md"
        w="100%"
        h="100%"
        justify="start"
        styles={{
          label: { height: "fit-content", color: "var(--mantine-color-dimmed-0)" },
          inner: { alignItems: "start" },
        }}
        onClick={() => setOpen(true)}
      >
        {resource?.description || "Set Description"}
      </Button>
    </>
  );
};
