import type { Headcount } from './enemy-tracker';

export const HeadcountLine = ({ allies, enemies, lasts, objective }: Headcount) => (
  <div class={`tracker-line ${allies > enemies ? 'ahead' : 'behind'}`}>
    <span class="tracker-lead">
      {allies}v{enemies}
    </span>{' '}
    {lasts}
    {objective && (
      <span class="tracker-detail">
        {' · '}
        {objective.objective} {objective.spawnsIn === null ? 'up' : `in ${objective.spawnsIn}`}
      </span>
    )}
  </div>
);
