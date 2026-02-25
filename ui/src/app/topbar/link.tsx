import { Button, ButtonProps } from "@mantine/core";
import { Link } from "react-router-dom";

export interface TopbarLinkProps extends ButtonProps {
  to: string;
}

export default function TopbarLink({ to, ...props }: TopbarLinkProps) {
  return (
    <Button
      visibleFrom="md"
      variant="subtle"
      px="xs"
      fz="sm"
      className="hover-underline"
      renderRoot={(props) => <Link to={to} target="_blank" {...props} />}
      {...props}
    />
  );
}
