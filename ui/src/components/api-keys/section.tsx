import { ICONS } from "@/theme/icons";
import Section, { SectionProps } from "@/ui/section";
import NewApiKey from "./new";
import ApiKeysTable from "./table";
import { useInvalidate, useManageAuth, useRead, useWrite } from "@/lib/hooks";
import { notifications } from "@mantine/notifications";
import { Box } from "@mantine/core";

export interface ApiKeysSectionProps extends SectionProps {
  /** For service user api keys */
  userId?: string;
}

export default function ApiKeysSection({
  userId,
  ...sectionProps
}: ApiKeysSectionProps) {
  const { data: keys, isPending } = useRead(
    userId ? "ListApiKeysForServiceUser" : "ListApiKeys",
    userId
      ? {
          user: userId,
        }
      : {},
  );
  const inv = useInvalidate();
  const { mutate: regularDelete, isPending: regularPending } = useManageAuth(
    "DeleteApiKey",
    {
      onSuccess: () => {
        inv(["ListApiKeys"]);
        notifications.show({ message: "Api key deleted.", color: "green" });
      },
    },
  );
  const { mutate: serviceDelete, isPending: servicePending } = useWrite(
    "DeleteApiKeyForServiceUser",
    {
      onSuccess: () => {
        inv(["ListApiKeysForServiceUser"]);
        notifications.show({ message: "Api key deleted.", color: "green" });
      },
    },
  );
  return (
    <Section
      isPending={isPending}
      title="Api Keys"
      titleFz="h3"
      titleMb="0"
      icon={<ICONS.Key size="1.2rem" />}
      titleRight={
        <Box ml="md">
          <NewApiKey userId={userId} />
        </Box>
      }
      withBorder
      {...sectionProps}
    >
      {keys && (
        <ApiKeysTable
          noBox
          keys={keys}
          onDelete={(key) =>
            userId ? serviceDelete({ key }) : regularDelete({ key })
          }
          deletePending={userId ? servicePending : regularPending}
        />
      )}
    </Section>
  );
}
