type MetricProps = {
  label: string;
  value: number | string;
};

export const Metric = ({ label, value }: MetricProps) => (
  <span class="metric">
    <span class="metric-label">{label}</span> {value}
  </span>
);
