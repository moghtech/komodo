import {
  createPolymorphicComponent,
  Group,
  MantineStyleProps,
  Stack,
  StackProps,
  Text,
} from "@mantine/core";
import { forwardRef, ReactNode } from "react";

// https://mantine.dev/guides/polymorphic/#create-your-own-polymorphic-components

export interface SectionProps extends StackProps {
  titleFz?: MantineStyleProps["fz"];
  titleNode?: ReactNode;
  icon?: ReactNode;
  titleRight?: ReactNode;
  titleOther?: ReactNode;
  description?: ReactNode;
  actions?: ReactNode;
  withBorder?: boolean;
  // otherwise items-start
  itemsCenterTitleRow?: boolean;
}

const Section = createPolymorphicComponent<"div", SectionProps>(
  forwardRef<HTMLDivElement, SectionProps>(
    (
      {
        titleFz = "h2",
        title,
        titleNode,
        icon,
        titleRight,
        titleOther,
        description,
        actions,
        children,
        withBorder,
        itemsCenterTitleRow,
        ...props
      },
      ref,
    ) => {
      const TitleComponent = ({ mb }: { mb?: MantineStyleProps["mb"] }) =>
        (title || titleNode || icon || titleRight || titleOther || actions) && (
          <Group
            align={itemsCenterTitleRow ? "center" : undefined}
            justify="space-between"
            mb={mb}
          >
            {title || titleNode || icon ? (
              <Group c="dimmed" gap="xs">
                {icon}
                {title && <Text fz={titleFz}>{title}</Text>}
                {titleNode}
                {titleRight}
              </Group>
            ) : (
              titleOther
            )}
            {actions}
          </Group>
        );
      return (
        <Stack
          px={withBorder ? "lg" : undefined}
          pt={withBorder ? "sm" : undefined}
          pb={withBorder ? "lg" : undefined}
          bdrs="md"
          style={(theme) => ({
            borderColor: theme.colors["accent-border"][0],
            borderStyle: "solid",
            borderWidth: withBorder ? 1 : 0,
          })}
          {...props}
          ref={ref}
        >
          {description ? (
            <Stack gap="0.2rem" mb="md">
              <TitleComponent />
              {description}
            </Stack>
          ) : (
            <TitleComponent mb="md" />
          )}
          {children}
        </Stack>
      );
    },
  ),
);

export default Section;
