import { Center, CenterProps, Loader, Text } from "@mantine/core";
import { ReactNode } from "react";

export interface PageGuardProps extends CenterProps {
  isPending?: boolean;
  error?: false | string;
  children?: ReactNode;
}

export default function PageGuard({
  isPending,
  error,
  children,
  ...centerProps
}: PageGuardProps) {
  if (isPending) {
    return (
      <Center h="30vh" {...centerProps}>
        <Loader size="xl" />
      </Center>
    );
  }

  if (error) {
    return (
      <Center h="30vh" {...centerProps}>
        <Text>{error}</Text>
      </Center>
    );
  }

  return children;
}
