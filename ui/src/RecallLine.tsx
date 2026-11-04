import type { ShownRecall } from './enemy-tracker';

export const RecallLine = ({ champion, backIn, teleport, bought }: ShownRecall) => (
  <div class="tracker-line">
    <span class="tracker-label">Base</span> {champion} · back ~{backIn}
    {teleport && <span class="tracker-detail"> · TP</span>}
    {bought.length > 0 && <div class="tracker-detail tracker-bought">{bought.join(', ')}</div>}
  </div>
);
