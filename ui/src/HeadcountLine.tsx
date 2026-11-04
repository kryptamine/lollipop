import type { ContestedObjective, Headcount } from './enemy-tracker';

const OBJECTIVE: Record<ContestedObjective, string> = {
  drake: 'Drake',
  elder: 'Elder',
  baron: 'Baron',
};

export const HeadcountLine = ({ allies, enemies, leadEndsIn, objective }: Headcount) => (
  <div class={`tracker-line ${allies > enemies ? 'ahead' : 'behind'}`}>
    <span class="tracker-lead">
      {allies}v{enemies}
    </span>{' '}
    {leadEndsIn}
    {objective && (
      <span class="tracker-detail">
        {' · '}
        {OBJECTIVE[objective.objective]}{' '}
        {objective.spawnsIn === null ? 'up' : `in ${objective.spawnsIn}`}
      </span>
    )}
  </div>
);
