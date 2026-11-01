// The only module that may import Tauri; components stay unaware of it.
import { listen } from '@tauri-apps/api/event';

export type Snapshot = {
  cs: number;
  level: number;
  goldEarned: number | null;
};

export const onSnapshot = (handler: (snapshot: Snapshot) => void): (() => void) => {
  const unlisten = listen<Snapshot>('snapshot', ({ payload }) => {
    handler(payload);
  });

  const stopListening = async () => {
    const stop = await unlisten;

    stop();
  };

  return () => {
    void stopListening();
  };
};
