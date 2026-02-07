import { ICONS } from "@/theme/icons";
import { Button, ButtonProps } from "@mantine/core";
import { Link } from "react-router-dom";

export interface BackButtonProps extends ButtonProps {
  to?: string | number;
}

export default function BackButton({ to = -1, ...props }: BackButtonProps) {
  return (
    <Button
      color="accent"
      leftSection={<ICONS.Back className="w-4" />}
      renderRoot={(props) => <Link to={to} {...props} />}
      {...props}
    >
      Back
    </Button>
  );
}
