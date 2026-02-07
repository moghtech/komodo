import { Group, Stack, StackProps } from "@mantine/core";
import BackButton from "./back-button";
import { ReactNode } from "react";

export interface EntityPageProps extends StackProps {
  backTo?: string;
  actions?: ReactNode;
}

export default function EntityPage({
  backTo,
  actions,
  children,
  ...props
}: EntityPageProps) {
  return (
    <Stack gap="lg" {...props}>
      <Group justify="space-between" mb="lg">
        <BackButton to={backTo} />
        {actions && <Group>{actions}</Group>}
      </Group>
      {children}
    </Stack>
  );
}
