import { Marker } from './Marker';
import type { MarkerState } from './bridge';

type MetricProps = {
  label: string;
  value: number | string;
  marker: MarkerState | null;
  benchmark: number | null;
};

export const Metric = ({ label, value, marker, benchmark }: MetricProps) => (
  <span class="metric">
    <span class="metric-label">{label}</span> {value} <Marker state={marker} />
    {benchmark !== null && <span class="metric-benchmark"> {Math.round(benchmark)}</span>}
  </span>
);
