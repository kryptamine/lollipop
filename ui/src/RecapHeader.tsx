import type { Recap } from './recap-types';
import { clock, scoreLine } from './format';

type RecapHeaderProps = {
  recap: Recap;
};

export const RecapHeader = ({
  recap: { champion, role, length, outcome, scores },
}: RecapHeaderProps) => (
  <header class="recap-header">
    <div>
      <div class="recap-title">{[champion, role].filter((part) => part !== null).join(' · ')}</div>
      <div class="recap-subtitle">
        {[clock(length), scores && `K/D/A ${scoreLine(scores)}`].filter(Boolean).join(' · ')}
      </div>
    </div>
    {outcome && <div class="recap-outcome">{outcome}</div>}
  </header>
);
