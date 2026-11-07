import type { Scores } from './scores';

export type RecapMinute = {
  minute: number;
  cs: number;
  goldEarned: number | null;
  // Null wherever the Marker strip shows no Benchmark, as for a support's CS.
  csBenchmark: number | null;
  goldBenchmark: number | null;
};

// A span of game time, in seconds.
export type Stretch = {
  from: number;
  to: number;
};

// Why the Lessons were not asked for.
export type LessonsOffReason =
  'noKey' | 'keyRefused' | 'adviceWidgetOff' | 'switchedOff' | 'recapOff';

type LessonsOff = {
  state: 'off';
  detail: LessonsOffReason;
};

type LessonsAsking = {
  state: 'asking';
};

type LessonsGiven = {
  state: 'given';
  // Most costly first.
  detail: string[];
};

type LessonsEmpty = {
  state: 'empty';
};

type LessonsFailed = {
  state: 'failed';
};

export type Lessons = LessonsOff | LessonsAsking | LessonsGiven | LessonsEmpty | LessonsFailed;

type Role = 'Top' | 'Jungle' | 'Mid' | 'Bot' | 'Support';

export type Recap = {
  champion: string;
  role: Role | null;
  // Null when no poll saw the game's end, which alone carries it.
  outcome: 'Victory' | 'Defeat' | null;
  // Game time at the end, in seconds.
  length: number;
  scores: Scores | null;
  minutes: RecapMinute[];
  deaths: number[];
  slowFarm: Stretch[];
  lessons: Lessons;
};
