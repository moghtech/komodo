import { Badge, Button, Group } from "@mantine/core";
import { Spotlight, spotlight } from "@mantine/spotlight";
import { useOmniSearch } from "./hooks";
import { ICONS } from "@/lib/icons";

export default function OmniSearch({}: {}) {
  const { search, setSearch, actions } = useOmniSearch();
  return (
    <>
      <Button
        // variant="default"
        // variant="light"
        color="accent.9"
        c="inherit"
        justify="space-between"
        rightSection={
          <Badge
            color="accent"
            tt="lowercase"
            c="dimmed"
            style={{ cursor: "pointer" }}
          >
            shift + s
          </Badge>
        }
        onClick={() => spotlight.open()}
        w={{ lg: 400 }}
      >
        <Group>
          <ICONS.Search size="1rem" />
          Search
        </Group>
      </Button>

      <Spotlight.Root
        query={search}
        onQueryChange={setSearch}
        clearQueryOnClose={false}
        shortcut="shift + S"
      >
        <Spotlight.Search
          leftSection={<ICONS.Search size="1.3rem" />}
          placeholder="Search..."
        />
        <Spotlight.ActionsList>
          {actions.map((group) => (
            <Spotlight.ActionsGroup key={group.group} label={group.group}>
              {group.actions.map((action) => (
                <Spotlight.Action key={action.id} {...action} />
              ))}
            </Spotlight.ActionsGroup>
          ))}
          {actions.length === 0 && (
            <Spotlight.Empty>Nothing found...</Spotlight.Empty>
          )}
        </Spotlight.ActionsList>
      </Spotlight.Root>
    </>
  );
}
