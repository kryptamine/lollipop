// The only module that may import Tauri; components stay unaware of it.
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type Snapshot = {
  cs: number;
  level: number;
  goldEarned: number | null;
};

export type MarkerState = 'above' | 'below' | 'neutral';

export type Markers = {
  cs: MarkerState | null;
  gold: MarkerState | null;
  level: MarkerState | null;
};

export type TimingHintState = {
  objective: string;
  countdown: string;
};

export type Notices = string[];

export type WidgetSwitches = {
  markerStrip: boolean;
  timingHints: boolean;
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

export const onMarkers = (handler: (markers: Markers) => void): (() => void) =>
  stopWith(
    listen<Markers>('markers', ({ payload }) => {
      handler(payload);
    }),
  );

export const onTimingHints = (handler: (hints: TimingHintState[]) => void): (() => void) =>
  stopWith(
    listen<TimingHintState[]>('timing-hints', ({ payload }) => {
      handler(payload);
    }),
  );

export const onNotices = (handler: (notices: Notices) => void): (() => void) =>
  stopWith(
    listen<Notices>('notices', ({ payload }) => {
      handler(payload);
    }),
  );

export const onWidgetSwitches = (handler: (switches: WidgetSwitches) => void): (() => void) =>
  stopWith(
    listen<WidgetSwitches>('widget-switches', ({ payload }) => {
      handler(payload);
    }),
  );
