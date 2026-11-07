type ChartLegendProps = {
  hasSlowFarm: boolean;
};

export const ChartLegend = ({ hasSlowFarm }: ChartLegendProps) => (
  <div class="chart-legend">
    <span class="legend-you">You</span>
    <span class="legend-usual">Usual for your champion, role and tier</span>
    {hasSlowFarm && <span class="legend-slow">Slow farm</span>}
    <span>✕ Death</span>
  </div>
);
