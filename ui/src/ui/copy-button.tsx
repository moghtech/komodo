import { ActionIcon, CopyButton as MantineCopyButton } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { Check, Copy } from "lucide-react";
import { ReactNode } from "react";

export interface CopyButtonProps {
  content: string;
  icon?: ReactNode;
  label?: string;
}

export default function CopyButton({
  content,
  icon,
  label = "content",
}: CopyButtonProps) {
  return (
    <MantineCopyButton value={content}>
      {({ copied, copy }) => (
        <ActionIcon
          variant="default"
          onClick={() => {
            copy();
            if (location.origin.startsWith("https")) {
              notifications.show({ message: `Copied ${label} to clipboard.` });
            } else {
              notifications.show({
                message: "Cannot copy to clipboard without HTTPS.",
                color: "red",
              });
            }
          }}
        >
          {copied ? <Check size="1rem" /> : (icon ?? <Copy size="1rem" />)}
        </ActionIcon>
      )}
    </MantineCopyButton>
  );
}
