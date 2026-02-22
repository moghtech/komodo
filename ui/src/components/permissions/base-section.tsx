import { Types } from "komodo_client";

type UpdateFn = (
  resource_type: Types.ResourceTarget["type"],
  permission: Types.PermissionLevelAndSpecifics,
) => void;

export interface BasePermissionsSectionProps {
  all: Types.User["all"];
  update: UpdateFn;
}

export default function BasePermissionsSection({}: BasePermissionsSectionProps) {}
