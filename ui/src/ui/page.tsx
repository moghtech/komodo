import {
  createPolymorphicComponent,
  Group,
  Paper,
  Stack,
  StackProps,
  Text,
} from "@mantine/core";
import { CircleQuestionMark } from "lucide-react";
import { FC, forwardRef, ReactNode } from "react";

// https://mantine.dev/guides/polymorphic/#create-your-own-polymorphic-components

export interface PageProps extends StackProps {
  title?: string;
  icon?: FC<{ size?: string | number }>;
  description?: ReactNode;
  rightTitle?: ReactNode;
  aboveTitle?: ReactNode;
  /** The right hand side of page opposite the title */
  oppositeTitle?: ReactNode;
  /* Replace the default title / icon with a full custom ReactNode */
  customTitle?: ReactNode;
  customDescription?: ReactNode;
  actions?: ReactNode;
  children?: ReactNode;
}

const Page = createPolymorphicComponent<"div", PageProps>(
  forwardRef<HTMLDivElement, PageProps>(
    (
      {
        title,
        icon,
        description,
        rightTitle,
        aboveTitle,
        customTitle,
        customDescription,
        oppositeTitle,
        actions,
        children,
        ...stackProps
      },
      ref,
    ) => {
      const Icon = icon ?? CircleQuestionMark;
      const titleNode = (
        <Paper
          w={{ base: "100%", xs: "fit-content" }}
          px="xl"
          py="xs"
          style={{
            border: "1px solid var(--mantine-color-accent-border-0)",
          }}
        >
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
            <Group gap="md" c="dimmed">
              {customDescription}
            </Group>
          ) : (
            description && (
              <Text size="md" c="dimmed">
                {description}
              </Text>
            )
          )}
        </Paper>
      );
      return (
        <Stack {...stackProps} ref={ref}>
          {aboveTitle}
          {oppositeTitle ? (
            <Group justify="space-between" align="start">
              {titleNode}
              {oppositeTitle}
            </Group>
          ) : (
            titleNode
          )}

          {actions && <Group gap="sm">{actions}</Group>}

          {children}
        </Stack>
      );
    },
  ),
);

export default Page;
