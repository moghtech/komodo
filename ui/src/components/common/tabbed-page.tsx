import { Center, Group, Tabs, Text } from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { FC } from "react";
import { Page } from "@/components/common/page";
import { ICONS } from "@/lib/icons";

export type TabbedPageItem<Tab extends string> = {
  tab: Tab;
  icon?: FC<{ size?: string | number }>;
  content?: FC;
};

export type TabbedPageProps<Tab extends string> = {
  /** Store current tab on localStorage */
  storageKey: string;
  tabs: TabbedPageItem<Tab>[];
};

export function TabbedPage<Tab extends string>({
  storageKey,
  tabs,
}: TabbedPageProps<Tab>) {
  const defaultTab = tabs[0]?.tab;
  const [selectedTab, setSelectedTab] = useLocalStorage<Tab>({
    key: storageKey,
    defaultValue: defaultTab,
  });
  const Content =
    tabs.find((tab) => tab.tab === selectedTab)?.content ??
    (() => (
      <Center>
        <ICONS.Unknown size={22} />
      </Center>
    ));
  return (
    <Tabs
      value={selectedTab}
      onChange={(tab) => setSelectedTab((tab as Tab) ?? defaultTab)}
    >
      <Page
        customTitle={
          <Tabs.List>
            {tabs.map(({ tab, icon }) => {
              const Icon = icon ?? ICONS.Unknown;
              return (
                <Tabs.Tab key={tab} value={tab}>
                  <Group opacity={tab === selectedTab ? 1 : 0.6}>
                    <Icon size={20} />
                    <Text fz="h3">{tab}</Text>
                  </Group>
                </Tabs.Tab>
              );
            })}
          </Tabs.List>
        }
      >
        <Content />
      </Page>
    </Tabs>
  );
}
