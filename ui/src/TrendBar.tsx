import type { TrendGame } from './recap-types';
import type { ValueScale } from './chart-geometry';

// A game's column on the chart, in SVG units.
type Column = {
  left: number;
  width: number;
};

type TrendBarProps = {
  game: TrendGame;
  column: Column;
  scale: ValueScale;
  isLatest: boolean;
};

export const TrendBar = ({
  game: { cs, benchmark },
  column: { left, width },
  scale,
  isLatest,
}: TrendBarProps) => {
  const top = scale.y(cs);

  return (
    <g class={isLatest ? 'trend-bar is-latest' : 'trend-bar'}>
      <rect x={left} y={top} width={width} height={scale.y(0) - top} rx={2} />
      {benchmark !== null && (
        <line
          class="tick-usual"
          x1={left}
          x2={left + width}
          y1={scale.y(benchmark)}
          y2={scale.y(benchmark)}
        />
      )}
      <text class="trend-value" x={left + width / 2} y={top + 13} text-anchor="middle">
        {String(cs)}
      </text>
    </g>
  );
};
