import { Metric } from './Metric';
import type { Markers, Snapshot } from './bridge';

type MarkerStripProps = {
  snapshot: Snapshot;
  markers: Markers;
};

export const MarkerStrip = ({
  snapshot: { cs, goldEarned, level },
  markers: { cs: csMarker, gold: goldMarker, level: levelMarker },
}: MarkerStripProps) => (
  <div id="marker-strip">
    <Metric label="CS" value={cs} marker={csMarker} />
    <Metric label="Gold" value={goldEarned ?? '—'} marker={goldMarker} />
    <Metric label="Level" value={level} marker={levelMarker} />
  </div>
);
