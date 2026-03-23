import { ICONS } from "@/theme/icons";
import {
  Box,
  Group,
  GroupProps,
  MantineBreakpoint,
  Select,
  Stack,
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
    <Tabs value={value} {...tabsProps}>
      {Selector}
      <Box mt="xl">
        <MobileFriendlyTabsContent tabs={tabs} value={value} />
      </Box>
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
  changeAt: _changeAt,
  fullIconSize = "1.1rem",
  mobileIconSize = "1rem",
  tabProps,
}: MobileFriendlyTabsSelectorProps) {
  const tabs = _tabs.filter((t) => !t.hidden);
  const changeAt =
    _changeAt ?? (tabs.length > 6 ? "lg" : tabs.length > 3 ? "md" : "sm");
  const SelectedIcon = tabs.find((tab) => tab.value === value)?.icon;
  return (
    <>
      {/* DESKTOP VIEW */}
      <Group justify="space-between" visibleFrom={changeAt}>
        <Tabs.List>
          {tabs.map(({ value: tabValue, label, icon: Icon, disabled }) => (
            <Tabs.Tab
              key={tabValue}
              value={tabValue}
              disabled={disabled}
              onClick={() => onValueChange(tabValue)}
              w="fit-content"
            >
              <Group
                gap="xs"
                fz="lg"
                justify="center"
                c={tabValue === value ? undefined : "dimmed"}
                w={100}
                miw="fit-content"
                wrap="nowrap"
                {...tabProps}
              >
                {Icon && <Icon size={fullIconSize} />}
                {label ?? tabValue}
              </Group>
            </Tabs.Tab>
          ))}
        </Tabs.List>
        {actions}
      </Group>

      {/* MOBILE VIEW */}
      <Stack hiddenFrom={changeAt} w="100%">
        <Select
          w={{ base: "100%", md: 300 }}
          value={value}
          onChange={(value) => value && onValueChange(value)}
          leftSection={SelectedIcon && <SelectedIcon size="1rem" />}
          data={tabs.map((tab) => ({
            value: tab.value,
            label: tab.label ?? tab.value,
            disabled: tab.disabled,
          }))}
          renderOption={({ option, checked }) => {
            const Icon = tabs.find((tab) => tab.value === option.value)?.icon;
            return (
              <Group gap="xs" p="0.25rem" {...tabProps}>
                {Icon && <Icon size={mobileIconSize} />}
                {option.label}
                {checked && <ICONS.Check size="1rem" />}
              </Group>
            );
          }}
          withScrollArea={false}
          styles={{
            dropdown: { maxHeight: "calc(100vh - 230px)", overflowY: "auto" },
          }}
        />
        {actions}
      </Stack>
    </>
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
