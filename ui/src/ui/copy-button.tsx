import { ActionIcon, CopyButton as MantineCopyButton } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { Check, Copy } from "lucide-react";

export const CopyButton = ({ content }: { content: string }) => {
  return (
    <MantineCopyButton value={content}>
      {({ copied, copy }) => (
        <ActionIcon
          variant="default"
          onClick={() => {
            copy();
            if (location.origin.startsWith("https")) {
              notifications.show({ message: "Copied content to clipboard." });
            } else {
              notifications.show({
                message: "Cannot copy to clipboard without HTTPS.",
                color: "red",
              });
            }
          }}
        >
          {copied ? <Check size="1rem" /> : <Copy size="1rem" />}
        </ActionIcon>
      )}
    </MantineCopyButton>
  );
};
