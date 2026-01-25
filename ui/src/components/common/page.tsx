import { Flex, Group, Paper, Text } from "@mantine/core";
import { CircleQuestionMark } from "lucide-react";
import { FC, ReactNode } from "react";

export type PageProps = {
  title?: string;
  icon?: FC<{ size?: string | number }>;
  description?: string;
  rightTitle?: ReactNode;
  /* Replace the default title / icon with a full custom ReactNode */
  customTitle?: ReactNode;
  customDescription?: ReactNode;
  actions?: ReactNode;
  children?: ReactNode;
};

export const Page = ({
  title,
  icon,
  description,
  rightTitle,
  customTitle,
  customDescription,
  actions,
  children,
}: PageProps) => {
  const Icon = icon ?? CircleQuestionMark;
  return (
    <Flex direction="column" gap="lg">
      <Paper w="fit-content" px="xl" py="xs" mb="lg" withBorder>
        <Group gap="sm">
          {customTitle ? (
            customTitle
          ) : (
            <>
              <Icon size={22} />
              <Text fz="h2">{title}</Text>
            </>
          )}
          {rightTitle}
        </Group>
        {customDescription ? (
          <Group gap="xs">{customDescription}</Group>
        ) : (
          description && (
            <Text size="md" opacity={0.6}>
              {description}
            </Text>
          )
        )}
      </Paper>

      {actions && <Group gap="sm">{actions}</Group>}

      {children}
    </Flex>
  );
};
