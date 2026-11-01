import { useEffect, useState } from 'preact/hooks';

import { MarkerStrip } from './MarkerStrip';
import { TimingHintBanner } from './TimingHintBanner';
import {
  onMarkers,
  onSnapshot,
  onTimingHints,
  onWidgetSwitches,
  type Markers,
  type Snapshot,
  type TimingHintState,
  type WidgetSwitches,
} from './bridge';

export const App = () => {
  const [snapshot, setSnapshot] = useState<Snapshot>();
  const [markers, setMarkers] = useState<Markers>();
  const [hints, setHints] = useState<TimingHintState[]>([]);
  const [switches, setSwitches] = useState<WidgetSwitches>();

  useEffect(() => onSnapshot(setSnapshot), []);
  useEffect(() => onMarkers(setMarkers), []);
  useEffect(() => onTimingHints(setHints), []);
  useEffect(() => onWidgetSwitches(setSwitches), []);

  return (
    <>
      {snapshot && markers && switches?.markerStrip && (
        <MarkerStrip snapshot={snapshot} markers={markers} />
      )}
      {switches?.timingHints && <TimingHintBanner hints={hints} />}
    </>
  );
};
