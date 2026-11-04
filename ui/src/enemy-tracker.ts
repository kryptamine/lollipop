// What the app sends the Enemy tracker Widget on each poll of a running game.

// An objective to take while one side has more champions alive; `spawnsIn` is null while it is up.
export type WindowObjective = {
  objective: string;
  spawnsIn: string | null;
};

// Champions alive on each side, only while one has more. The lead holds at least `lasts`.
export type Headcount = {
  allies: number;
  enemies: number;
  lasts: string;
  objective: WindowObjective | null;
};

export type JunglerPlace =
  'top' | 'mid' | 'bot' | 'jungle' | 'base' | 'drake' | 'elder' | 'herald' | 'baron';

export type JunglerDead = {
  state: 'dead';
  backIn: string;
};

export type JunglerSeen = {
  state: 'seen';
  place: JunglerPlace;
  ago: string;
};

export type JunglerSighting = {
  champion: string;
  whereabouts: JunglerDead | JunglerSeen;
};

// A Lane opponent back in base, and what they bought there.
export type ShownRecall = {
  champion: string;
  ago: string;
  bought: string[];
};

export type EnemyTracker = {
  headcount: Headcount | null;
  jungler: JunglerSighting | null;
  recalls: ShownRecall[];
};
