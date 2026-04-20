import ReactECharts from "echarts-for-react";

interface Props {
  title: string;
  dates: string[];
  values: (number | null)[];
  color?: string;
  unit?: string;
}

export default function LineChart({ title, dates, values, color = "#428a6a", unit = "" }: Props) {
  const option = {
    title: { text: title, left: 8, top: 6, textStyle: { fontSize: 14, fontWeight: 600 } },
    grid: { left: 40, right: 18, top: 40, bottom: 28 },
    tooltip: { trigger: "axis", valueFormatter: (v: number) => `${v} ${unit}` },
    xAxis: { type: "category", data: dates, axisLine: { lineStyle: { color: "#b9d9c9" } } },
    yAxis: { type: "value", scale: true, splitLine: { lineStyle: { color: "#eef4f1" } } },
    series: [
      {
        data: values,
        type: "line",
        smooth: true,
        showSymbol: true,
        lineStyle: { width: 3, color },
        itemStyle: { color },
        areaStyle: { color: color + "22" },
      },
    ],
  };
  return <ReactECharts option={option} style={{ height: 260, width: "100%" }} />;
}
