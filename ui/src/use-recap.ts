import { useEffect, useState } from 'preact/hooks';

import type { Recap } from './recap-types';
import { onRecap, readRecap } from './bridge';

// The last game's Recap, `undefined` until one exists, then each update the app sends.
export const useRecap = (): Recap | undefined => {
  const [recap, setRecap] = useState<Recap>();

  useEffect(() => {
    const showKept = async () => {
      const kept = await readRecap();

      // An event that came first is newer than what was kept.
      setRecap((shown) => shown ?? kept);
    };

    void showKept();

    return onRecap(setRecap);
  }, []);

  return recap;
};
