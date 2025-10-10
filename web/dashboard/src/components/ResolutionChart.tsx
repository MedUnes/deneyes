import dayjs from "dayjs";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  ResponsiveContainer,
  Tooltip,
  Legend,
  CartesianGrid
} from "recharts";
import { DnsResolutionEvent } from "../types";

type ResolutionChartProps = {
  data: DnsResolutionEvent[];
};

export function ResolutionChart({ data }: ResolutionChartProps) {
  const chartData = data
    .slice()
    .sort(
      (a, b) => new Date(a.finished_at).getTime() - new Date(b.finished_at).getTime()
    )
    .map((event) => ({
      time: dayjs(event.finished_at).format("HH:mm"),
      duration: event.duration_ms,
      success: event.success ? 1 : 0
    }));

  return (
    <div className="card chart-card">
      <h2>Latency and status</h2>
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={chartData} margin={{ top: 16, right: 24, left: 0, bottom: 0 }}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="time" minTickGap={32} />
          <YAxis yAxisId="left" label={{ value: "ms", angle: -90, position: "insideLeft" }} />
          <YAxis
            yAxisId="right"
            orientation="right"
            domain={[0, 1]}
            ticks={[0, 1]}
            label={{ value: "Success", angle: -90, position: "insideRight" }}
          />
          <Tooltip />
          <Legend />
          <Line
            yAxisId="left"
            type="monotone"
            dataKey="duration"
            stroke="#2563eb"
            strokeWidth={2}
            dot={false}
            name="Latency"
          />
          <Line
            yAxisId="right"
            type="stepAfter"
            dataKey="success"
            stroke="#16a34a"
            strokeWidth={2}
            dot={false}
            name="Success"
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
