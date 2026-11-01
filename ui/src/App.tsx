import { useEffect, useState } from 'preact/hooks';

import { MarkerStrip } from './MarkerStrip';
import {
  onMarkers,
  onSnapshot,
  onWidgetSwitches,
  type Markers,
  type Snapshot,
  type WidgetSwitches,
} from './bridge';

export const App = () => {
  const [snapshot, setSnapshot] = useState<Snapshot>();
  const [markers, setMarkers] = useState<Markers>();
  const [switches, setSwitches] = useState<WidgetSwitches>();

  useEffect(() => onSnapshot(setSnapshot), []);
  useEffect(() => onMarkers(setMarkers), []);
  useEffect(() => onWidgetSwitches(setSwitches), []);

  return (
    snapshot &&
    markers &&
    switches?.markerStrip && <MarkerStrip snapshot={snapshot} markers={markers} />
  );
};
