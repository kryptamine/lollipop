import { Metric } from './Metric';
import { Notices } from './Notices';
import { MetricsHead } from './MetricsHead';
import { scoreLine, tenths, whole } from './format';
import type { Benchmarks, Markers, Notices as NoticesState, Snapshot } from './bridge';

type MarkerStripProps = {
  snapshot: Snapshot;
  markers: Markers;
  benchmarks: Benchmarks;
  notices: NoticesState;
};

export const MarkerStrip = ({
  snapshot: { cs, goldEarned, level, scores, csPace },
  markers: {
    cs: csMarker,
    gold: goldMarker,
    level: levelMarker,
    scores: scoresMarker,
    csPace: csPaceMarker,
  },
  benchmarks: {
    cs: csBenchmark,
    gold: goldBenchmark,
    level: levelBenchmark,
    scores: scoresBenchmark,
    csPace: csPaceBenchmark,
  },
  notices,
}: MarkerStripProps) => (
  <div id="marker-strip">
    <table class="metrics">
      <MetricsHead />
      <tbody>
        {scores && (
          <Metric
            label="K / D / A"
            value={scoreLine(scores)}
            marker={scoresMarker}
            benchmark={scoresBenchmark ? scoreLine(scoresBenchmark) : undefined}
          />
        )}
        <Metric label="CS" value={cs} marker={csMarker} benchmark={whole(csBenchmark)} />
        <Metric
          label="CS / min"
          value={tenths(csPace) ?? '—'}
          marker={csPaceMarker}
          benchmark={tenths(csPaceBenchmark)}
        />
        <Metric
          label="Gold earned"
          value={goldEarned ?? '—'}
          marker={goldMarker}
          benchmark={whole(goldBenchmark)}
        />
        <Metric
          label="Level"
          value={level}
          marker={levelMarker}
          benchmark={whole(levelBenchmark)}
        />
      </tbody>
    </table>
    {notices.length > 0 && <Notices notices={notices} />}
  </div>
);
