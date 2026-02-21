import { useUser } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import ConfirmButton from "@/ui/confirm-button";
import ConfirmModal from "@/ui/confirm-modal";
import { DataTable } from "@/ui/data-table";
import StatusBadge from "@/ui/status-badge";
import { Badge, BoxProps } from "@mantine/core";
import { ColumnDef } from "@tanstack/react-table";
import { Types } from "komodo_client";
import { useNavigate } from "react-router-dom";

export interface UserTableProps extends BoxProps {
  users: Types.User[];
  onUserRemove?: (user_id: string) => void;
  onUserDelete?: (user_id: string) => Promise<unknown>;
  userDeleteDisabled?: (user_id: string) => boolean;
  onSelfClick?: () => void;
}

export default function UserTable({
  users,
  onUserRemove,
  onUserDelete,
  userDeleteDisabled,
  onSelfClick,
  ...boxProps
}: UserTableProps) {
  const user = useUser().data;
  const nav = useNavigate();

  const columns: ColumnDef<Types.User, "User" | "Admin" | "Super Admin">[] = [
    { header: "Username", accessorKey: "username" },
    {
      header: "Type",
      accessorKey: "config.type",
      cell: ({ row }) => <Badge>{row.original.config.type}</Badge>,
    },
    {
      header: "Level",
      accessorFn: (user) =>
        user.admin ? (user.super_admin ? "Super Admin" : "Admin") : "User",
    },
    {
      header: "Enabled",
      cell: ({ row }) => {
        return (
          <StatusBadge
            text={row.original.enabled ? "Enabled" : "Disabled"}
            intent={row.original.enabled ? "Good" : "Critical"}
          />
        );
      },
    },
  ];

  if (onUserRemove) {
    columns.push({
      header: "Remove",
      cell: ({ row }) => (
        <ConfirmButton
          variant="filled"
          color="red"
          icon={<ICONS.Remove size="1rem" />}
          onClick={(e) => {
            e.stopPropagation();
            onUserRemove(row.original._id?.$oid!);
          }}
        >
          Remove
        </ConfirmButton>
      ),
    });
  }

  if (onUserDelete) {
    columns.push({
      header: "Delete",
      cell: ({ row }) => (
        <ConfirmModal
          targetProps={{ variant: "filled", color: "red" }}
          icon={<ICONS.Delete size="1rem" />}
          confirmText={row.original.username}
          onConfirm={() => onUserDelete(row.original._id?.$oid!)}
          disabled={
            row.original._id?.$oid
              ? (userDeleteDisabled?.(row.original._id.$oid) ?? true)
              : true
          }
        >
          Delete
        </ConfirmModal>
      ),
    });
  }

  return (
    <DataTable
      tableKey="user-table"
      data={users}
      columns={columns}
      onRowClick={(row) =>
        row._id?.$oid === user?._id?.$oid
          ? onSelfClick?.()
          : nav(`/users/${row._id!.$oid}`)
      }
      {...boxProps}
    />
  );
}
