import {
  Center,
  CenterProps,
  createPolymorphicComponent,
  Group,
  Loader,
  MantineStyleProps,
  Stack,
  StackProps,
  Text,
} from "@mantine/core";
import { forwardRef, ReactNode } from "react";

// https://mantine.dev/guides/polymorphic/#create-your-own-polymorphic-components

export interface SectionProps extends StackProps {
  titleFz?: MantineStyleProps["fz"];
  titleMb?: MantineStyleProps["mb"];
  titleDimmed?: boolean;
  titleNode?: ReactNode;
  icon?: ReactNode;
  titleRight?: ReactNode;
  titleOther?: ReactNode;
  description?: ReactNode;
  actions?: ReactNode;
  withBorder?: boolean;
  isPending?: boolean;
  error?: false | string;
  guardProps?: CenterProps;
  forceHeaderGroup?: boolean;
  onHeaderClick?: () => void;
}

const Section = createPolymorphicComponent<"div", SectionProps>(
  forwardRef<HTMLDivElement, SectionProps>(
    (
      {
        titleFz = "h2",
        titleMb,
        title,
        titleDimmed,
        titleNode,
        icon,
        titleRight,
        titleOther,
        description,
        actions,
        children,
        withBorder,
        isPending,
        error,
        guardProps,
        forceHeaderGroup,
        onHeaderClick,
        ...props
      },
      ref,
    ) => {
      const TitleComponentInner = (title ||
        titleNode ||
        icon ||
        titleRight ||
        titleOther ||
        actions) && (
        <>
          {title || titleNode || icon ? (
            <Group c={titleDimmed ? "dimmed" : undefined} gap="xs">
              {icon}
              {title && <Text fz={titleFz}>{title}</Text>}
              {titleNode}
              {titleRight}
            </Group>
          ) : (
            titleOther
          )}
          {actions}
        </>
      );

      const TitleComponent =
        TitleComponentInner &&
        (forceHeaderGroup ? (
          <Group justify="space-between">{TitleComponentInner}</Group>
        ) : (
          <>
            <Stack hiddenFrom="xs">{TitleComponentInner}</Stack>
            <Group visibleFrom="xs" justify="space-between">
              {TitleComponentInner}
            </Group>
          </>
        ));

      return (
        <Stack
          px={withBorder ? "lg" : undefined}
          pt={withBorder ? "sm" : undefined}
          pb={withBorder ? "lg" : undefined}
          bdrs="md"
          className={withBorder ? "bordered-light" : undefined}
          {...props}
          ref={ref}
        >
          <Stack
            gap="0.2rem"
            mb={titleMb}
            onClick={onHeaderClick}
            style={{ cursor: onHeaderClick && "pointer" }}
          >
            {TitleComponent}
            {description && <Text c="dimmed">{description}</Text>}
          </Stack>
          {isPending ? (
            <Center {...guardProps}>
              <Loader size="xl" />
            </Center>
          ) : error ? (
            <Center {...guardProps}>
              <Text>{error}</Text>
            </Center>
          ) : (
            children
          )}
        </Stack>
      );
    },
  ),
);

export default Section;
