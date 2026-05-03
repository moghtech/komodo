import { ButtonLink, ButtonLinkProps } from "mogh_ui";

export interface TopbarLinkProps extends ButtonLinkProps {
  to: string;
}

export default function TopbarLink({ to, ...props }: TopbarLinkProps) {
  return <ButtonLink visibleFrom="md" to={to} {...props} />;
}
