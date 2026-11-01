import { Metric } from './Metric';
import type { Snapshot } from './bridge';

type MarkerStripProps = {
  snapshot: Snapshot;
};

export const MarkerStrip = ({ snapshot: { cs, goldEarned, level } }: MarkerStripProps) => (
  <div id="marker-strip">
    <Metric label="CS" value={cs} />
    <Metric label="Gold" value={goldEarned ?? '—'} />
    <Metric label="Level" value={level} />
  </div>
);
