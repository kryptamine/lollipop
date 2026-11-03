import { useEffect, useState } from 'preact/hooks';

import { MarkerStrip } from './MarkerStrip';
import { AdviceWidget } from './AdviceWidget';
import { TimingHintBanner } from './TimingHintBanner';
import {
  onAdvice,
  onBenchmarks,
  onMarkers,
  onNotices,
  onSnapshot,
  onTimingHints,
  onWidgetSwitches,
  type Benchmarks,
  type Markers,
  type Notices,
  type ShownAdvice,
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
  const [switches, setSwitches] = useState<WidgetSwitches>();

  useEffect(() => onSnapshot(setSnapshot), []);
  useEffect(() => onMarkers(setMarkers), []);
  useEffect(() => onBenchmarks(setBenchmarks), []);
  useEffect(() => onNotices(setNotices), []);
  useEffect(() => onTimingHints(setHints), []);
  useEffect(() => onAdvice(setAdvice), []);
  useEffect(() => onWidgetSwitches(setSwitches), []);

  return (
    <>
      {snapshot && markers && benchmarks && switches?.markerStrip && (
        <MarkerStrip
          snapshot={snapshot}
          markers={markers}
          benchmarks={benchmarks}
          notices={notices}
        />
      )}
      {switches?.timingHints && <TimingHintBanner hints={hints} />}
      {switches?.adviceWidget && <AdviceWidget advice={advice} />}
    </>
  );
};
