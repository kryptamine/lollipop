import { useEffect, useState } from 'preact/hooks';

import { MarkerStrip } from './MarkerStrip';
import { onSnapshot, onWidgetSwitches, type Snapshot, type WidgetSwitches } from './bridge';

export const App = () => {
  const [snapshot, setSnapshot] = useState<Snapshot>();
  const [switches, setSwitches] = useState<WidgetSwitches>();

  useEffect(() => onSnapshot(setSnapshot), []);
  useEffect(() => onWidgetSwitches(setSwitches), []);

  return snapshot && switches?.markerStrip && <MarkerStrip snapshot={snapshot} />;
};
