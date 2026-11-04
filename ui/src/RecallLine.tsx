import type { ShownRecall } from './enemy-tracker';

export const RecallLine = ({ champion, ago, bought }: ShownRecall) => (
  <div class="tracker-line">
    <span class="tracker-label">Base</span> {champion} · {ago} ago
    {bought.length > 0 && <div class="tracker-detail tracker-bought">{bought.join(', ')}</div>}
  </div>
);
