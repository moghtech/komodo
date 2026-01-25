import { Group, Paper, Stack, Text } from "@mantine/core";
import { PieChart } from "react-minimal-pie-chart";
import { ReactNode, useMemo } from "react";
import { ColorIntention, hexColorByIntention } from "@/lib/color";
import classes from "./index.module.scss";

export type PieChartItem = {
  title: string;
  intention: ColorIntention;
  value: number;
};

export default function DashboardSummary({
  name,
  icon,
  onClick,
  data: _data,
  children: _children,
}: {
  name: string;
  icon: ReactNode;
  onClick: () => void;
  data?: Array<PieChartItem | false | undefined>;
  children?: ReactNode;
}) {
  const data = useMemo(
    () => _data?.filter((d) => d) ?? [],
    [_data],
  ) as Array<PieChartItem>;
  const children = _children ? (
    _children
  ) : (
    <Group gap="md" wrap="nowrap">
      <Stack w={120} gap="0.2rem">
        {data.map(({ title, value, intention }) => (
          <Group key={title} gap="sm" fz="sm" opacity={0.6}>
            <Text fw="bold" c={hexColorByIntention(intention)}>
              {value}
            </Text>
            {title}
          </Group>
        ))}
      </Stack>
      <PieChart
        style={{ width: 150, height: 150 }}
        radius={42}
        lineWidth={30}
        data={data.map(({ title, value, intention }) => ({
          title,
          value,
          color: hexColorByIntention(intention) || "#A855F7",
        }))}
      />
    </Group>
  );
  return (
    <Paper className={classes["dashboard-summary"]} onClick={onClick}>
      <Group>
        {icon}
        {name}s
      </Group>
      {children}
    </Paper>
  );
}
