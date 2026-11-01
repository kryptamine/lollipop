import { useEffect, useState } from 'preact/hooks';

import { MarkerStrip } from './MarkerStrip';
import { TimingHintBanner } from './TimingHintBanner';
import {
  onMarkers,
  onNotices,
  onSnapshot,
  onTimingHints,
  onWidgetSwitches,
  type Markers,
  type Notices,
  type Snapshot,
  type TimingHintState,
  type WidgetSwitches,
} from './bridge';

export const App = () => {
  const [snapshot, setSnapshot] = useState<Snapshot>();
  const [markers, setMarkers] = useState<Markers>();
  const [notices, setNotices] = useState<Notices>([]);
  const [hints, setHints] = useState<TimingHintState[]>([]);
  const [switches, setSwitches] = useState<WidgetSwitches>();

  useEffect(() => onSnapshot(setSnapshot), []);
  useEffect(() => onMarkers(setMarkers), []);
  useEffect(() => onNotices(setNotices), []);
  useEffect(() => onTimingHints(setHints), []);
  useEffect(() => onWidgetSwitches(setSwitches), []);

  return (
    <>
      {snapshot && markers && switches?.markerStrip && (
        <MarkerStrip snapshot={snapshot} markers={markers} notices={notices} />
      )}
      {switches?.timingHints && <TimingHintBanner hints={hints} />}
    </>
  );
};
