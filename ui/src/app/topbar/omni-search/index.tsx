import { ActionIcon, Badge, Button, Group } from "@mantine/core";
import { Spotlight, spotlight } from "@mantine/spotlight";
import { useOmniSearch } from "./hooks";
import { ICONS } from "@/theme/icons";
import { useShiftKeyListener } from "@/lib/hooks";

export default function OmniSearch({}: {}) {
  const { search, setSearch, actions } = useOmniSearch();
  useShiftKeyListener("S", () => spotlight.open());
  return (
    <>
      <ActionIcon
        variant="subtle"
        onClick={() => spotlight.open()}
        hiddenFrom="lg"
        size="xl"
      >
        <ICONS.Search size="1.3rem" />
      </ActionIcon>

      <Button
        justify="space-between"
        rightSection={
          <Badge color="accent.9" tt="lowercase" style={{ cursor: "pointer" }}>
            shift + s
          </Badge>
        }
        onClick={() => spotlight.open()}
        w={{ lg: 230, xl: 300, xl4: 400 }}
        visibleFrom="lg"
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
      >
        <Spotlight.Search
          leftSection={<ICONS.Search size="1.3rem" />}
          placeholder="search..."
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
