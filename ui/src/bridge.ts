// The only module that may import Tauri; components stay unaware of it.
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type Scores = {
  kills: number;
  deaths: number;
  assists: number;
};

export type Snapshot = {
  cs: number;
  level: number;
  goldEarned: number | null;
  scores: Scores | null;
};

export type MarkerState = 'above' | 'below' | 'neutral';

export type Markers = {
  cs: MarkerState | null;
  gold: MarkerState | null;
  level: MarkerState | null;
  scores: MarkerState | null;
};

export type Benchmarks = {
  cs: number | null;
  gold: number | null;
  level: number | null;
  scores: Scores | null;
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

export type WidgetId = keyof WidgetSwitches;

export type Switches = {
  launchAtLogin: boolean;
  widgets: WidgetSwitches;
};

// Each answers with the switches as kept, which may differ from what was asked.
export const readSwitches = (): Promise<Switches> => invoke<Switches>('read_switches');

export const setWidget = (id: WidgetId, isOn: boolean): Promise<Switches> =>
  invoke<Switches>('set_widget', {
    id,
    isOn,
  });

export const setLaunchAtLogin = (isOn: boolean): Promise<Switches> =>
  invoke<Switches>('set_launch_at_login', { isOn });

export type KeyFailure = 'rejected' | 'unreachable' | 'notKept';

export type KeySaved = {
  outcome: 'saved';
  masked: string;
};

export type KeyRefused = {
  outcome: KeyFailure;
};

export type KeySave = KeySaved | KeyRefused;

// The app answers a saved key masked as `sk-…abcd`, never in full, and no key as null.
export const readOpenAiKey = async (): Promise<string | undefined> =>
  (await invoke<string | null>('read_openai_key')) ?? undefined;

export const saveOpenAiKey = (key: string): Promise<KeySave> =>
  invoke<KeySave>('save_openai_key', { key });

export const removeOpenAiKey = async (): Promise<string | undefined> =>
  (await invoke<string | null>('remove_openai_key')) ?? undefined;

export const openOpenAiUsage = (): Promise<void> => invoke('open_openai_usage');

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

export const onBenchmarks = (handler: (benchmarks: Benchmarks) => void): (() => void) =>
  stopWith(
    listen<Benchmarks>('benchmarks', ({ payload }) => {
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
