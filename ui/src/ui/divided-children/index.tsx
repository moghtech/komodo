import { createPolymorphicComponent, Group, GroupProps } from "@mantine/core";
import { forwardRef } from "react";
import classes from "./index.module.scss";

// https://mantine.dev/guides/polymorphic/#create-your-own-polymorphic-components

const DividedChildren = createPolymorphicComponent<"div", GroupProps>(
  forwardRef<HTMLDivElement, GroupProps>(({ className, ...props }, ref) => (
    <Group
      className={
        className
          ? classes["divided-children"] + " " + className
          : classes["divided-children"]
      }
      gap="md"
      {...props}
      ref={ref}
    />
  )),
);

export default DividedChildren;
