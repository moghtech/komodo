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
    <Stack gap="xl" mb="50vh" {...props}>
      <Group justify="space-between" mb="lg">
        <BackButton to={backTo} />
        {actions && <Group wrap="nowrap">{actions}</Group>}
      </Group>
      {children}
    </Stack>
  );
}
