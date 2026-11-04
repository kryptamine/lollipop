import type { JunglerPlace, JunglerSighting } from './enemy-tracker';

const PLACE: Record<JunglerPlace, string> = {
  top: 'top',
  mid: 'mid',
  bot: 'bot',
  jungle: 'in your jungle',
  base: 'in base',
  drake: 'at Drake',
  elder: 'at Elder',
  herald: 'at Herald',
  baron: 'at Baron',
};

export const JunglerLine = ({ champion, whereabouts }: JunglerSighting) => (
  <div class="tracker-line">
    <span class="tracker-label">Jungler</span> {champion}{' '}
    {whereabouts.state === 'dead'
      ? `dead · back in ${whereabouts.backIn}`
      : `${PLACE[whereabouts.place]} · ${whereabouts.ago} ago`}
  </div>
);
