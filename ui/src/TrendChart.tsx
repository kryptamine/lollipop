import { TrendBar } from './TrendBar';
import type { TrendGame } from './recap-types';
import { HEIGHT, LEFT, RIGHT, WIDTH, valueScaleOf } from './chart-geometry';

// The share of each game's slot left empty between bars.
const GAP = 0.2;

type TrendChartProps = {
  champion: string;
  trend: TrendGame[];
};

export const TrendChart = ({ champion, trend }: TrendChartProps) => {
  const scale = valueScaleOf(trend.flatMap(({ cs, benchmark }) => [cs, benchmark]));
  const slot = (WIDTH - LEFT - RIGHT) / trend.length;
  const title = `CS at 10:00 · last ${String(trend.length)} games on ${champion}`;

  return (
    <section class="chart">
      <div class="chart-head">
        <div class="caption">{title}</div>
        <div class="chart-legend">
          <span class="legend-bar">You</span>
          <span class="legend-usual">Usual at 10:00</span>
        </div>
      </div>
      <div class="chart-plot">
        <svg viewBox={`0 0 ${String(WIDTH)} ${String(HEIGHT)}`} role="img" aria-label={title}>
          {scale.ticks.map((tick) => (
            <g key={tick}>
              <line
                class="grid"
                x1={LEFT}
                x2={WIDTH - RIGHT}
                y1={scale.y(tick)}
                y2={scale.y(tick)}
              />
              <text class="axis" x={LEFT - 6} y={scale.y(tick) + 4} text-anchor="end">
                {String(tick)}
              </text>
            </g>
          ))}
          {trend.map((game, index) => (
            <TrendBar
              // eslint-disable-next-line @eslint-react/no-array-index-key -- a game has no id, and its place in the Trend is its identity
              key={index}
              game={game}
              column={{
                left: LEFT + slot * (index + GAP / 2),
                width: slot * (1 - GAP),
              }}
              scale={scale}
              isLatest={index === trend.length - 1}
            />
          ))}
        </svg>
      </div>
    </section>
  );
};
