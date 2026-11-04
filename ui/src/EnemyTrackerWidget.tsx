import { RecallLine } from './RecallLine';
import { JunglerLine } from './JunglerLine';
import { HeadcountLine } from './HeadcountLine';
import type { EnemyTracker } from './enemy-tracker';

type EnemyTrackerWidgetProps = {
  // Nothing shows before the first poll of a game.
  tracker: EnemyTracker | undefined;
};

export const EnemyTrackerWidget = ({ tracker }: EnemyTrackerWidgetProps) => {
  if (!tracker) {
    return;
  }

  const { headcount, jungler, recalls } = tracker;

  if (!headcount && !jungler && recalls.length === 0) {
    return;
  }

  return (
    <div id="enemy-tracker">
      {headcount && <HeadcountLine {...headcount} />}
      {jungler && <JunglerLine champion={jungler.champion} whereabouts={jungler.whereabouts} />}
      {recalls.map(({ champion, ago, bought }) => (
        <RecallLine key={champion} champion={champion} ago={ago} bought={bought} />
      ))}
    </div>
  );
};
