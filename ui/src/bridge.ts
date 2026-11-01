// The only module that may import Tauri; components stay unaware of it.
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type Snapshot = {
  cs: number;
  level: number;
  goldEarned: number | null;
};

export type WidgetSwitches = {
  markerStrip: boolean;
};

const stopWith = (unlisten: Promise<UnlistenFn>): (() => void) => {
  const stopListening = async () => {
    const stop = await unlisten;

    stop();
  };

  return () => {
    void stopListening();
  };
};

export const onSnapshot = (handler: (snapshot: Snapshot) => void): (() => void) =>
  stopWith(
    listen<Snapshot>('snapshot', ({ payload }) => {
      handler(payload);
    }),
  );

export const onWidgetSwitches = (handler: (switches: WidgetSwitches) => void): (() => void) =>
  stopWith(
    listen<WidgetSwitches>('widget-switches', ({ payload }) => {
      handler(payload);
    }),
  );
