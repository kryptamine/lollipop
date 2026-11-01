import { useEffect, useState } from 'preact/hooks';

import { MarkerStrip } from './MarkerStrip';
import { onSnapshot, type Snapshot } from './bridge';

export const App = () => {
  const [snapshot, setSnapshot] = useState<Snapshot>();

  useEffect(() => onSnapshot(setSnapshot), []);

  return snapshot && <MarkerStrip snapshot={snapshot} />;
};
