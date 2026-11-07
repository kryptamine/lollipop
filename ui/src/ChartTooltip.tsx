import type { ChartPoint } from './chart-geometry';

type ChartTooltipProps = {
  point: ChartPoint;
  // Where the crosshair is, as a share of the chart's width.
  share: number;
};

export const ChartTooltip = ({ point: { minute, value, benchmark }, share }: ChartTooltipProps) => (
  <div
    class="chart-tooltip"
    style={share > 0.6 ? { right: `${(1 - share) * 100}%` } : { left: `${share * 100}%` }}
  >
    <div class="tooltip-minute">{`${String(minute)}:00`}</div>
    <div>{`You ${String(value)}`}</div>
    {benchmark !== null && <div>{`Usual ${String(Math.round(benchmark))}`}</div>}
  </div>
);
