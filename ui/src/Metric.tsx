import { Marker } from './Marker';
import type { MarkerState } from './bridge';

type MetricProps = {
  label: string;
  value: number | string;
  marker: MarkerState | null;
  benchmark: number | null;
};

export const Metric = ({ label, value, marker, benchmark }: MetricProps) => (
  <tr>
    <th class="metric-label" scope="row">
      {label}
    </th>
    <td>
      <Marker state={marker} />
    </td>
    <td>{value}</td>
    <td class="metric-benchmark">{benchmark !== null && Math.round(benchmark)}</td>
  </tr>
);
