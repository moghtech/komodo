import { useInvalidate, usePermissions, useRead, useWrite } from "@/lib/hooks";
import { UsableResource } from ".";
import { notifications } from "@mantine/notifications";
import { TextUpdateModal } from "mogh_ui";
import { Button, Text } from "@mantine/core";
import { fmtUpperCamelcase } from "mogh_ui";

export default function ResourceDescription({
  type,
  id,
}: {
  type: UsableResource;
  id: string;
}) {
  const { canWrite } = usePermissions({ type, id });
  const inv = useInvalidate();
  const key = type === "ResourceSync" ? "sync" : type.toLowerCase();

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
    <TextUpdateModal
      title={`Update ${fmtUpperCamelcase(type)} Description`}
      placeholder="Set Description"
      value={resource?.description}
      onUpdate={(description) =>
        updateDescription({ target: { type, id }, description })
      }
      disabled={!canWrite}
      target={(open) => (
        <Button
          variant="transparent"
          c="dimmed"
          p="md"
          className="bordered-light"
          bdrs="md"
          w="100%"
          h="100%"
          justify="start"
          styles={{
            label: {
              height: "fit-content",
              color: "var(--mantine-color-dimmed-0)",
            },
            inner: { alignItems: "start" },
          }}
          onClick={open}
        >
          <Text
            className="text-ellipsis"
            maw={{ xl: 600, xl2: 750, xl3: 900, xl4: 1050 }}
          >
            {resource?.description || "Set Description"}
          </Text>
        </Button>
      )}
    />
  );
}
