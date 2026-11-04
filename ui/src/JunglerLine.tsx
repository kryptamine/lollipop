import type { JunglerDead, JunglerSeen, JunglerSighting, Place } from './enemy-tracker';

const PLACE: Record<Place, string> = {
  top: 'top',
  mid: 'mid',
  bot: 'bot',
  jungle: 'in your jungle',
  base: 'in base',
  drake: 'at Drake',
  elder: 'at Elder',
  voidgrubs: 'at Voidgrubs',
  herald: 'at Herald',
  baron: 'at Baron',
};

const whereaboutsText = (whereabouts: JunglerDead | JunglerSeen): string =>
  whereabouts.state === 'dead'
    ? `dead · back in ${whereabouts.backIn}`
    : `${PLACE[whereabouts.place]} · ${whereabouts.ago} ago`;

export const JunglerLine = ({ champion, whereabouts, safeLane, noFarmFor }: JunglerSighting) => (
  <div class="tracker-line">
    <span class="tracker-label">Jungler</span> {champion}
    {whereabouts && ` ${whereaboutsText(whereabouts)}`}
    {safeLane && (
      <div class="tracker-detail">
        {safeLane.lane} safe {safeLane.endsIn}
      </div>
    )}
    {noFarmFor !== null && <div class="tracker-detail">no farm {noFarmFor}</div>}
  </div>
);
