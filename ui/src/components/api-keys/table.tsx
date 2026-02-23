import { ICONS } from "@/theme/icons";
import ConfirmButton from "@/ui/confirm-button";
import CopyText from "@/ui/copy-text";
import { DataTable } from "@/ui/data-table";
import { Text } from "@mantine/core";
import { Types } from "komodo_client";

const ONE_DAY_MS = 1000 * 60 * 60 * 24;

export interface ApiKeysTableProps {
  keys: Types.ApiKey[];
  onDelete: (key: string) => void;
  deletePending: boolean;
  noBox?: boolean;
}

export default function ApiKeysTable({
  keys,
  onDelete,
  deletePending,
  noBox,
}: ApiKeysTableProps) {
  return (
    <DataTable
      noBox={noBox}
      tableKey="api-keys"
      data={keys}
      columns={[
        { header: "Name", accessorKey: "name" },
        {
          header: "Key",
          cell: ({
            row: {
              original: { key },
            },
          }) => {
            return <CopyText content={key} label="api key" />;
          },
        },
        {
          header: "Expires",
          cell: ({ row }) => {
            return (
              <Text>
                {row.original.expires ? (
                  <>
                    In{" "}
                    <b>
                      {(
                        (row.original.expires - Date.now()) /
                        ONE_DAY_MS
                      ).toFixed()}
                    </b>{" "}
                    Days
                  </>
                ) : (
                  <b>Never</b>
                )}
              </Text>
            );
          },
        },
        {
          header: "Delete",
          cell: ({ row }) => (
            <ConfirmButton
              variant="filled"
              color="red"
              icon={<ICONS.Delete size="1rem" />}
              onClick={() => onDelete(row.original.key)}
              loading={deletePending}
            >
              Delete
            </ConfirmButton>
          ),
        },
      ]}
    />
  );
}
