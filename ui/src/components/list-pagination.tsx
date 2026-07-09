import { Group, Pagination } from "@mantine/core";

/** Matches the server side default page size on `List<Resource>` calls. */
export const RESOURCE_PAGE_SIZE = 100;

/**
 * Pagination controls for the paginated `List<Resource>` calls.
 * Only renders when needed, ie the results fill the page
 * or the user is past the first page.
 */
export default function ListPagination({
  page,
  setPage,
  count,
}: {
  page: number;
  setPage: (page: number) => void;
  count: number;
}) {
  if (count < RESOURCE_PAGE_SIZE && page === 0) return null;
  return (
    <Pagination.Root
      total={count >= RESOURCE_PAGE_SIZE ? page + 2 : page + 1}
      value={page + 1}
      onChange={(page) => setPage(page - 1)}
    >
      <Group gap="0.2rem" justify="center">
        <Pagination.First />
        <Pagination.Previous />
        <Pagination.Items />
        <Pagination.Next />
      </Group>
    </Pagination.Root>
  );
}
