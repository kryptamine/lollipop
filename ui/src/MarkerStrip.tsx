import { Metric } from './Metric';
import { Notices } from './Notices';
import { scoreLine, whole } from './format';
import type { Benchmarks, Markers, Notices as NoticesState, Snapshot } from './bridge';

type MarkerStripProps = {
  snapshot: Snapshot;
  markers: Markers;
  benchmarks: Benchmarks;
  notices: NoticesState;
};

export const MarkerStrip = ({
  snapshot: { cs, goldEarned, level, scores },
  markers: { cs: csMarker, gold: goldMarker, level: levelMarker, scores: scoresMarker },
  benchmarks: {
    cs: csBenchmark,
    gold: goldBenchmark,
    level: levelBenchmark,
    scores: scoresBenchmark,
  },
  notices,
}: MarkerStripProps) => (
  <div id="marker-strip">
    <table class="metrics">
      <thead>
        <tr>
          <th scope="col">Stat</th>
          <td />
          <th scope="col">Current</th>
          <th scope="col">Benchmark</th>
        </tr>
      </thead>
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
