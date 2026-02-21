import { sendCopyNotification } from "@/lib/utils";
import {
  ActionIcon,
  ActionIconProps,
  CopyButton as MantineCopyButton,
} from "@mantine/core";
import { Check, Copy } from "lucide-react";
import { ReactNode } from "react";

export interface CopyButtonProps {
  content: string;
  icon?: ReactNode;
  label?: string;
  size?: string | number;
  buttonSize?: ActionIconProps["size"];
}

export default function CopyButton({
  content,
  icon,
  label = "content",
  size = "1.1rem",
  buttonSize = "lg",
}: CopyButtonProps) {
  return (
    <MantineCopyButton value={content}>
      {({ copied, copy }) => (
        <ActionIcon
          variant="default"
          onClick={(e) => {
            e.stopPropagation();
            copy();
            sendCopyNotification(label);
          }}
          size={buttonSize}
        >
          {copied ? <Check size={size} /> : (icon ?? <Copy size={size} />)}
        </ActionIcon>
      )}
    </MantineCopyButton>
  );
}
