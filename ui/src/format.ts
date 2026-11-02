import type { Scores } from './bridge';

export const whole = (value: number | null): string | undefined =>
  value === null ? undefined : String(Math.round(value));

export const scoreLine = ({ kills, deaths, assists }: Scores): string =>
  [kills, deaths, assists].map((count) => Math.round(count)).join(' / ');
