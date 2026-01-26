import {
  createPolymorphicComponent,
  Group,
  Stack,
  StackProps,
  Text,
} from "@mantine/core";
import { forwardRef, ReactNode } from "react";

// https://mantine.dev/guides/polymorphic/#create-your-own-polymorphic-components

interface SectionProps extends StackProps {
  titleNode?: ReactNode;
  icon?: ReactNode;
  titleRight?: ReactNode;
  titleOther?: ReactNode;
  actions?: ReactNode;
  withBorder?: boolean;
  // otherwise items-start
  itemsCenterTitleRow?: boolean;
}

const Section = createPolymorphicComponent<"div", SectionProps>(
  forwardRef<HTMLDivElement, SectionProps>(
    (
      {
        title,
        titleNode,
        icon,
        titleRight,
        titleOther,
        actions,
        children,
        withBorder,
        itemsCenterTitleRow,
        ...props
      },
      ref,
    ) => (
      <Stack
        gap="md"
        px="lg"
        pt="sm"
        pb="lg"
        bdrs="md"
        style={(theme) => ({
          borderColor: theme.colors["accent-border"][0],
          borderStyle: "solid",
          borderWidth: withBorder ? 1 : 0,
        })}
        {...props}
        ref={ref}
      >
        {(title ||
          titleNode ||
          icon ||
          titleRight ||
          titleOther ||
          actions) && (
          <Group
            align={itemsCenterTitleRow ? "center" : undefined}
            justify="space-between"
            mb="md"
          >
            {title || titleNode || icon ? (
              <Group c="dimmed" gap="xs">
                {icon}
                {title && <Text fz="h2">{title}</Text>}
                {titleNode}
                {titleRight}
              </Group>
            ) : (
              titleOther
            )}
            {actions}
          </Group>
        )}
        {children}
      </Stack>
    ),
  ),
);

export default Section;
