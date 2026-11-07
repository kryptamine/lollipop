import { ChartGrid } from './ChartGrid';
import { ChartEvents } from './ChartEvents';
import { ChartLegend } from './ChartLegend';
import type { Stretch } from './recap-types';
import { ChartTooltip } from './ChartTooltip';
import { useHoveredPoint } from './use-hovered-point';
import {
  BOTTOM,
  HEIGHT,
  TOP,
  WIDTH,
  pathOf,
  scaleOf,
  toMinutes,
  type ChartPoint,
} from './chart-geometry';

type RecapChartProps = {
  title: string;
  points: ChartPoint[];
  deaths: number[];
  slowFarm: Stretch[];
  // Game time at the end, in seconds.
  length: number;
};

export const RecapChart = ({ title, points, deaths, slowFarm, length }: RecapChartProps) => {
  const lastMinute = toMinutes(length);
  const scale = scaleOf(points, lastMinute);
  const { hovered, handlePointerMove, handlePointerLeave } = useHoveredPoint(points, lastMinute);
  const usual = points.map(({ minute, benchmark }) => ({
    minute,
    value: benchmark,
  }));

  return (
    <section class="chart">
      <div class="chart-head">
        <div class="caption">{title}</div>
        <ChartLegend hasSlowFarm={slowFarm.length > 0} />
      </div>
      <div class="chart-plot">
        <svg
          viewBox={`0 0 ${String(WIDTH)} ${String(HEIGHT)}`}
          role="img"
          aria-label={`${title} by minute against games like it`}
          onPointerMove={handlePointerMove}
          onPointerLeave={handlePointerLeave}
        >
          <ChartGrid scale={scale} lastMinute={lastMinute} />
          <ChartEvents scale={scale} deaths={deaths} slowFarm={slowFarm} />
          <path class="line-usual" d={pathOf(scale, usual)} />
          <path class="line-you" d={pathOf(scale, points)} />
          {hovered && (
            <line
              class="crosshair"
              x1={scale.x(hovered.minute)}
              x2={scale.x(hovered.minute)}
              y1={TOP}
              y2={HEIGHT - BOTTOM}
            />
          )}
        </svg>
        {hovered && <ChartTooltip point={hovered} share={scale.x(hovered.minute) / WIDTH} />}
      </div>
    </section>
  );
};
