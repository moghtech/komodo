import { ICONS } from "@/lib/icons";
import {
  Group,
  GroupProps,
  MantineBreakpoint,
  Select,
  Tabs,
  TabsProps,
} from "@mantine/core";
import { FC, ReactNode } from "react";

export interface Tab {
  label?: string;
  icon?: FC<{ size?: string | number }>;
  hidden?: boolean;
  disabled?: boolean;
  value: string;
  content: ReactNode;
}

export type TabNoContent = Omit<Tab, "content">;

export interface MobileFriendlyTabsProps extends MobileFriendlyTabsSelectorProps {
  tabs: Tab[];
  tabsProps?: Omit<TabsProps, "value">;
}

export default function MobileFriendlyTabs(props: MobileFriendlyTabsProps) {
  return (
    <MobileFriendlyTabsWrapper
      Selector={<MobileFriendlyTabsSelector {...props} />}
      tabs={props.tabs}
      value={props.value}
      {...props.tabsProps}
    />
  );
}

export interface MobileFriendlyTabsWrapper extends TabsProps {
  Selector: ReactNode;
  tabs: Tab[];
  value: string;
}

export function MobileFriendlyTabsWrapper({
  Selector,
  tabs,
  value,
  children,
  ...tabsProps
}: MobileFriendlyTabsWrapper) {
  return (
    <Tabs {...tabsProps}>
      {Selector}
      <MobileFriendlyTabsContent tabs={tabs} value={value} />
    </Tabs>
  );
}

export interface MobileFriendlyTabsSelectorProps {
  tabs: TabNoContent[];
  actions?: ReactNode;
  value: string;
  onValueChange: (value: string) => void;
  changeAt?: MantineBreakpoint;
  fullIconSize?: string | number;
  mobileIconSize?: string | number;
  tabProps?: GroupProps;
}

export function MobileFriendlyTabsSelector({
  tabs: _tabs,
  actions,
  value,
  onValueChange,
  changeAt = "md",
  fullIconSize = "1.3rem",
  mobileIconSize = "1rem",
  tabProps,
}: MobileFriendlyTabsSelectorProps) {
  const tabs = _tabs.filter((t) => !t.hidden);
  return (
    <Group>
      {/* DESKTOP VIEW */}
      <Tabs.List visibleFrom={changeAt}>
        {tabs.map(({ value, label, icon: Icon, disabled }) => (
          <Tabs.Tab
            key={value}
            value={value}
            disabled={disabled}
            onClick={() => onValueChange(value)}
            w="fit-content"
          >
            <Group gap="xs" fz="h3" justify="center" {...tabProps}>
              {Icon && <Icon size={fullIconSize} />}
              {label ?? value}
            </Group>
          </Tabs.Tab>
        ))}
      </Tabs.List>

      {/* MOBILE VIEW */}
      <Select
        hiddenFrom={changeAt}
        value={value}
        onChange={(value) => value && onValueChange(value)}
        data={tabs.map((tab) => ({
          value: tab.value,
          label: tab.label ?? tab.value,
          disabled: tab.disabled,
        }))}
        renderOption={({ option, checked }) => {
          const Icon = tabs.find((tab) => tab.value === option.value)?.icon;
          return (
            <Group gap="xs" justify="space-between" p="0.25rem">
              <Group gap="xs" {...tabProps}>
                {Icon && <Icon size={mobileIconSize} />}
                {option.label}
              </Group>
              {checked && <ICONS.Check size="1rem" />}
            </Group>
          );
        }}
      />

      {actions}
    </Group>
  );
}

export function MobileFriendlyTabsContent({
  tabs,
  value,
}: {
  tabs: Tab[];
  value: string;
}) {
  return tabs.find((tab) => tab.value === value)?.content;
}
