import { BOTTOM, HEIGHT, LEFT, RIGHT, WIDTH, type Scale } from './chart-geometry';

const MINUTE_STEP = 5;

type ChartGridProps = {
  scale: Scale;
  lastMinute: number;
};

export const ChartGrid = ({ scale, lastMinute }: ChartGridProps) => {
  const minutes = Array.from(
    { length: Math.floor(lastMinute / MINUTE_STEP) + 1 },
    (_, index) => index * MINUTE_STEP,
  );

  return (
    <>
      {scale.ticks.map((tick) => (
        <g key={tick}>
          <line class="grid" x1={LEFT} x2={WIDTH - RIGHT} y1={scale.y(tick)} y2={scale.y(tick)} />
          <text class="axis" x={LEFT - 6} y={scale.y(tick) + 4} text-anchor="end">
            {String(tick)}
          </text>
        </g>
      ))}
      {minutes.map((minute) => (
        <text
          key={minute}
          class="axis"
          x={scale.x(minute)}
          y={HEIGHT - BOTTOM + 14}
          text-anchor="middle"
        >
          {String(minute)}
        </text>
      ))}
    </>
  );
};
