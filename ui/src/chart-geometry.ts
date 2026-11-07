export type ChartPoint = {
  minute: number;
  value: number;
  benchmark: number | null;
};

// A point of a line, with no value where the line breaks.
export type LinePoint = {
  minute: number;
  value: number | null;
};

const SECONDS_PER_MINUTE = 60;

// Game times are in seconds and the chart's axis in minutes.
export const toMinutes = (seconds: number): number => seconds / SECONDS_PER_MINUTE;

export const WIDTH = 600;
export const HEIGHT = 150;
export const LEFT = 44;
export const RIGHT = 8;
export const TOP = 14;
export const BOTTOM = 20;

export type Scale = {
  x: (minute: number) => number;
  y: (value: number) => number;
  // Round values for four to six gridlines.
  ticks: number[];
};

const roundStep = (top: number): number => {
  const rough = top / 5;
  const power = 10 ** Math.floor(Math.log10(rough));

  return [1, 2, 2.5, 5, 10].map((factor) => factor * power).find((step) => step >= rough) ?? rough;
};

export const scaleOf = (points: ChartPoint[], lastMinute: number): Scale => {
  const highest = Math.max(
    1,
    ...points.flatMap(({ value, benchmark }) => (benchmark === null ? value : [value, benchmark])),
  );

  const step = roundStep(highest);
  const top = Math.ceil(highest / step) * step;
  const span = Math.max(1, lastMinute);

  return {
    x: (minute) => LEFT + (minute / span) * (WIDTH - LEFT - RIGHT),
    y: (value) => TOP + (1 - value / top) * (HEIGHT - TOP - BOTTOM),
    ticks: Array.from({ length: Math.round(top / step) + 1 }, (_, index) => index * step),
  };
};

// One path through the points it is given; a gap in benchmarks starts a new run.
export const pathOf = (scale: Scale, points: LinePoint[]): string =>
  points
    .map(({ minute, value }, index) => {
      if (value === null) {
        return '';
      }

      const isJoined = index > 0 && points[index - 1]?.value !== null;

      return `${isJoined ? 'L' : 'M'}${scale.x(minute).toFixed(1)},${scale.y(value).toFixed(1)}`;
    })
    .join('');
