import { useEffect, useState } from 'preact/hooks';

import { BuildWidget } from './BuildWidget';
import { MarkerStrip } from './MarkerStrip';
import { AdviceWidget } from './AdviceWidget';
import type { EnemyTracker } from './enemy-tracker';
import { TimingHintBanner } from './TimingHintBanner';
import { EnemyTrackerWidget } from './EnemyTrackerWidget';
import {
  onAdvice,
  onBenchmarks,
  onBuild,
  onBuildChange,
  onEnemyTracker,
  onMarkers,
  onNotices,
  onSnapshot,
  onSwap,
  onTimingHints,
  onWidgetSwitches,
  type Benchmarks,
  type BuildItem,
  type Markers,
  type Notices,
  type ShownAdvice,
  type ShownSwap,
  type Snapshot,
  type TimingHintState,
  type WidgetSwitches,
} from './bridge';

export const App = () => {
  const [snapshot, setSnapshot] = useState<Snapshot>();
  const [markers, setMarkers] = useState<Markers>();
  const [benchmarks, setBenchmarks] = useState<Benchmarks>();
  const [notices, setNotices] = useState<Notices>([]);
  const [hints, setHints] = useState<TimingHintState[]>([]);
  const [advice, setAdvice] = useState<ShownAdvice>();
  const [build, setBuild] = useState<BuildItem[]>([]);
  const [buildChange, setBuildChange] = useState<string>();
  const [swap, setSwap] = useState<ShownSwap>();
  const [tracker, setTracker] = useState<EnemyTracker>();
  const [switches, setSwitches] = useState<WidgetSwitches>();

  useEffect(() => onSnapshot(setSnapshot), []);
  useEffect(() => onMarkers(setMarkers), []);
  useEffect(() => onBenchmarks(setBenchmarks), []);
  useEffect(() => onNotices(setNotices), []);
  useEffect(() => onTimingHints(setHints), []);
  useEffect(() => onAdvice(setAdvice), []);
  useEffect(() => onBuild(setBuild), []);
  useEffect(() => onBuildChange(setBuildChange), []);
  useEffect(() => onSwap(setSwap), []);
  useEffect(() => onEnemyTracker(setTracker), []);
  useEffect(() => onWidgetSwitches(setSwitches), []);

  // No Widget draws until its switch is known.
  if (!switches) {
    return;
  }

  return (
    <>
      {snapshot && markers && benchmarks && switches.markerStrip && (
        <MarkerStrip
          snapshot={snapshot}
          markers={markers}
          benchmarks={benchmarks}
          notices={notices}
        />
      )}
      {switches.timingHints && <TimingHintBanner hints={hints} />}
      {switches.buildWidget && <BuildWidget build={build} change={buildChange} swap={swap} />}
      {switches.adviceWidget && <AdviceWidget advice={advice} />}
      {switches.enemyTracker && <EnemyTrackerWidget tracker={tracker} />}
    </>
  );
};
