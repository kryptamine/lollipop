import { Metric } from './Metric';
import { Notices } from './Notices';
import type { Markers, Notices as NoticesState, Snapshot } from './bridge';

type MarkerStripProps = {
  snapshot: Snapshot;
  markers: Markers;
  notices: NoticesState;
};

export const MarkerStrip = ({
  snapshot: { cs, goldEarned, level },
  markers: { cs: csMarker, gold: goldMarker, level: levelMarker },
  notices,
}: MarkerStripProps) => (
  <div id="marker-strip">
    <div class="metrics">
      <Metric label="CS" value={cs} marker={csMarker} />
      <Metric label="Gold earned" value={goldEarned ?? '—'} marker={goldMarker} />
      <Metric label="Level" value={level} marker={levelMarker} />
    </div>
    {notices.length > 0 && <Notices notices={notices} />}
  </div>
);
