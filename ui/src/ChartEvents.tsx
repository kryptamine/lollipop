import type { Stretch } from './recap-types';
import { BOTTOM, HEIGHT, TOP, toMinutes, type Scale } from './chart-geometry';

type ChartEventsProps = {
  scale: Scale;
  deaths: number[];
  slowFarm: Stretch[];
};

export const ChartEvents = ({ scale, deaths, slowFarm }: ChartEventsProps) => (
  <>
    {slowFarm.map(({ from, to }) => (
      <rect
        key={from}
        class="slow-farm"
        x={scale.x(toMinutes(from))}
        y={TOP}
        width={scale.x(toMinutes(to)) - scale.x(toMinutes(from))}
        height={HEIGHT - TOP - BOTTOM}
      />
    ))}
    {deaths.map((at) => (
      <text key={at} class="death" x={scale.x(toMinutes(at))} y={TOP - 3} text-anchor="middle">
        ✕
      </text>
    ))}
  </>
);
