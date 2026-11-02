import { Metric } from './Metric';
import { Notices } from './Notices';
import type { Benchmarks, Markers, Notices as NoticesState, Snapshot } from './bridge';

type MarkerStripProps = {
  snapshot: Snapshot;
  markers: Markers;
  benchmarks: Benchmarks;
  notices: NoticesState;
};

export const MarkerStrip = ({
  snapshot: { cs, goldEarned, level },
  markers: { cs: csMarker, gold: goldMarker, level: levelMarker },
  benchmarks: { cs: csBenchmark, gold: goldBenchmark, level: levelBenchmark },
  notices,
}: MarkerStripProps) => (
  <div id="marker-strip">
    <div class="metrics">
      <Metric label="CS" value={cs} marker={csMarker} benchmark={csBenchmark} />
      <Metric
        label="Gold earned"
        value={goldEarned ?? '—'}
        marker={goldMarker}
        benchmark={goldBenchmark}
      />
      <Metric label="Level" value={level} marker={levelMarker} benchmark={levelBenchmark} />
    </div>
    {notices.length > 0 && <Notices notices={notices} />}
  </div>
);
