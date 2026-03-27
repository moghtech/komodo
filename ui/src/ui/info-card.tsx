import {
  createPolymorphicComponent,
  Group,
  Stack,
  StackProps,
  Text,
  TextProps,
} from "@mantine/core";
import { forwardRef, ReactNode } from "react";

// https://mantine.dev/guides/polymorphic/#create-your-own-polymorphic-components

export interface InfoCardProps extends StackProps {
  title: string;
  titleProps?: TextProps;
  info?: ReactNode;
}

const InfoCard = createPolymorphicComponent<"div", InfoCardProps>(
  forwardRef<HTMLDivElement, InfoCardProps>(
    ({ title, info, titleProps, children, ...props }, ref) => {
      return (
        <Stack p="md" className="bordered-light" bdrs="md" {...props} ref={ref}>
          <Group justify="space-between">
            <Text fz="lg" {...titleProps}>
              {title}
            </Text>
            {info}
          </Group>
          {children}
        </Stack>
      );
    },
  ),
);

export default InfoCard;
