import { forwardRef } from "react";
import { Text, TextProps, createPolymorphicComponent } from "@mantine/core";
import {
  colorByIntention,
  ColorIntention,
  hexColorByIntention,
} from "@/lib/color";
import { fmtUpperCamelcase, snakeCaseToUpperSpaceCase } from "@/lib/formatting";

// https://mantine.dev/guides/polymorphic/#create-your-own-polymorphic-components

interface StatusBadgeProps extends TextProps {
  text: string | undefined;
  intent: ColorIntention;
}

const StatusBadge = createPolymorphicComponent<"div", StatusBadgeProps>(
  forwardRef<HTMLDivElement, StatusBadgeProps>(
    ({ text: _text, intent, ...props }, ref) => {
      if (!_text) {
        return null;
      }

      const text = snakeCaseToUpperSpaceCase(
        fmtUpperCamelcase(_text),
      ).toUpperCase();
      const color = colorByIntention(intent);
      const background = hexColorByIntention(intent) + "25";

      return (
        <Text
          fz="sm"
          bdrs="md"
          miw={30}
          w="fit-content"
          c={color}
          bg={background}
          px="0.5rem"
          py="0.2rem"
          lts="0.025em"
          {...props}
          ref={ref}
        >
          {text}
        </Text>
      );
    },
  ),
);

export default StatusBadge;
