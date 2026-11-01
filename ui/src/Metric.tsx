import { Marker } from './Marker';
import type { MarkerState } from './bridge';

type MetricProps = {
  label: string;
  value: number | string;
  marker: MarkerState | null;
};

export const Metric = ({ label, value, marker }: MetricProps) => (
  <span class="metric">
    <span class="metric-label">{label}</span> {value} <Marker state={marker} />
  </span>
);
