import type { RecapMinute } from './recap-types';
import type { ChartPoint } from './chart-geometry';

// One metric's reading at a minute, its value null where the Recap has none.
type Reading = {
  value: number | null;
  benchmark: number | null;
};

type MinuteReading = Reading & {
  minute: number;
};

const isCharted = (point: MinuteReading): point is ChartPoint => point.value !== null;

const pointsOf = (minutes: RecapMinute[], read: (minute: RecapMinute) => Reading): ChartPoint[] =>
  minutes
    .map((minute) => ({
      minute: minute.minute,
      ...read(minute),
    }))
    .filter(isCharted);

export const csPoints = (minutes: RecapMinute[]): ChartPoint[] =>
  pointsOf(minutes, ({ cs, csBenchmark }) => ({
    value: cs,
    benchmark: csBenchmark,
  }));

// A game first seen past its first minute has no Gold earned to chart.
export const goldPoints = (minutes: RecapMinute[]): ChartPoint[] =>
  pointsOf(minutes, ({ goldEarned, goldBenchmark }) => ({
    value: goldEarned,
    benchmark: goldBenchmark,
  }));
