import type { Scores } from './scores';

export const whole = (value: number | null): string | undefined =>
  value === null ? undefined : String(Math.round(value));

export const tenths = (value: number | null): string | undefined =>
  value === null ? undefined : value.toFixed(1);

export const scoreLine = ({ kills, deaths, assists }: Scores): string =>
  [kills, deaths, assists].map((count) => Math.round(count)).join(' / ');

// Game time as the client shows it, such as 24:05.
export const clock = (seconds: number): string => {
  const whole = Math.max(0, Math.floor(seconds));

  return `${String(Math.floor(whole / 60))}:${String(whole % 60).padStart(2, '0')}`;
};
