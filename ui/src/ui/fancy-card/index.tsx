import { forwardRef } from "react";
import { Box, BoxProps, createPolymorphicComponent } from "@mantine/core";
import classes from "./index.module.css";

// https://mantine.dev/guides/polymorphic/#create-your-own-polymorphic-components

interface FancyCardProps extends BoxProps {}

const FancyCard = createPolymorphicComponent<"div", FancyCardProps>(
  forwardRef<HTMLDivElement, FancyCardProps>(({ className, ...props }, ref) => (
    <Box
      className={
        className
          ? classes["fancy-card"] + " " + className
          : classes["fancy-card"]
      }
      {...props}
      ref={ref}
    />
  )),
);

export default FancyCard;
