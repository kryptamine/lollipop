// What the app sends the Enemy tracker Widget on each poll of a running game.

export type Place =
  'top' | 'mid' | 'bot' | 'jungle' | 'base' | 'drake' | 'elder' | 'voidgrubs' | 'herald' | 'baron';

export type ContestedObjective = 'drake' | 'elder' | 'baron';

// `spawnsIn` is null while the objective is up.
export type WindowObjective = {
  objective: ContestedObjective;
  spawnsIn: string | null;
};

// Champions alive on each side, only while one has more.
export type Headcount = {
  allies: number;
  enemies: number;
  leadEndsIn: string;
  objective: WindowObjective | null;
};

export type JunglerDead = {
  state: 'dead';
  backIn: string;
};

export type JunglerSeen = {
  state: 'seen';
  place: Place;
  ago: string;
};

// The side lane the enemy jungler cannot reach yet.
export type SafeLane = {
  lane: 'top' | 'bot';
  endsIn: string;
};

export type JunglerSighting = {
  champion: string;
  whereabouts: JunglerDead | JunglerSeen | null;
  safeLane: SafeLane | null;
  // Time since their CS or level last rose: ganking, or in base.
  noFarmFor: string | null;
};

// A Lane opponent in base; `backIn` estimates their walk back to lane.
export type ShownRecall = {
  champion: string;
  backIn: string;
  teleport: boolean;
  bought: string[];
};

export type EnemyTracker = {
  headcount: Headcount | null;
  jungler: JunglerSighting | null;
  recalls: ShownRecall[];
};
