import { Group, GroupProps, Text, TextProps } from "@mantine/core";
import CopyButton from "@/ui/copy-button";
import { sendCopyNotification } from "@/lib/utils";

export interface CopyTextProps extends TextProps {
  content: string;
  label?: string;
  groupProps?: GroupProps;
}

export default function CopyText({
  content,
  label,
  groupProps,
  ...textProps
}: CopyTextProps) {
  return (
    <Group gap="sm" wrap="nowrap" {...groupProps}>
      <Text
        title={content}
        w={{ base: 200, lg: 300 }}
        p="xs"
        bdrs="sm"
        style={{
          overflow: "hidden",
          whiteSpace: "nowrap",
          cursor: "pointer",
        }}
        className="text-ellipsis bordered-light"
        size="sm"
        onClick={() => {
          navigator.clipboard.writeText(content);
          sendCopyNotification(label);
        }}
        {...textProps}
      >
        {content}
      </Text>
      <CopyButton content={content} />
    </Group>
  );
}
