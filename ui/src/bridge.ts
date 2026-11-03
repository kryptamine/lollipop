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
  adviceWidget: boolean;
};

export type WidgetId = keyof WidgetSwitches;

export type TriggerSwitches = {
  loadingScreen: boolean;
  backToBase: boolean;
  turningPoint: boolean;
  death: boolean;
};

export type TriggerId = keyof TriggerSwitches;

export type AdviceLanguage = 'system' | 'en' | 'ru';

export type Switches = {
  launchAtLogin: boolean;
  widgets: WidgetSwitches;
  triggers: TriggerSwitches;
  adviceLanguage: AdviceLanguage;
  voiceover: boolean;
};

// What the Advice Widget shows; `timeLeft` runs from 1 down to 0 over its time on screen.
export type ShownAdvice = {
  trigger: TriggerId;
  text: string;
  // The advised item's icon on the Data Dragon CDN.
  icon: string | null;
  timeLeft: number;
};

// Each answers with the switches as kept, which may differ from what was asked.
export const readSwitches = (): Promise<Switches> => invoke<Switches>('read_switches');

export const setWidget = (id: WidgetId, isOn: boolean): Promise<Switches> =>
  invoke<Switches>('set_widget', {
    id,
    isOn,
  });

export const setTrigger = (id: TriggerId, isOn: boolean): Promise<Switches> =>
  invoke<Switches>('set_trigger', {
    id,
    isOn,
  });

export const setAdviceLanguage = (language: AdviceLanguage): Promise<Switches> =>
  invoke<Switches>('set_advice_language', { language });

export const setLaunchAtLogin = (isOn: boolean): Promise<Switches> =>
  invoke<Switches>('set_launch_at_login', { isOn });

export const setVoiceover = (isOn: boolean): Promise<Switches> =>
  invoke<Switches>('set_voiceover', { isOn });

// For the Advice language the app keeps now, so ask again once it changes.
export const isVoiceInstalled = (): Promise<boolean> => invoke<boolean>('read_voice_installed');

type SaveFailure = 'rejected' | 'unreachable' | 'notKept';

// Why OpenAI refused the saved key in a game; Advice stays off until a key is saved.
export type GameKeyFailure = 'rejected' | 'noQuota';

export type KeyFailure = SaveFailure | GameKeyFailure;

export type KeySaved = {
  outcome: 'saved';
  masked: string;
};

export type KeyRefused = {
  outcome: SaveFailure;
};

export type KeySave = KeySaved | KeyRefused;

// The app answers a saved key masked as `sk-…abcd`, never in full, and no key as null.
export const readOpenAiKey = async (): Promise<string | undefined> =>
  (await invoke<string | null>('read_openai_key')) ?? undefined;

export const readOpenAiKeyFailure = async (): Promise<GameKeyFailure | undefined> =>
  (await invoke<GameKeyFailure | null>('read_openai_key_failure')) ?? undefined;

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

// The app sends null while no Advice is showing.
export const onAdvice = (handler: (advice: ShownAdvice | undefined) => void): (() => void) =>
  stopWith(
    listen<ShownAdvice | null>('advice', ({ payload }) => {
      handler(payload ?? undefined);
    }),
  );

export const onOpenAiKeyFailure = (handler: (failure: GameKeyFailure) => void): (() => void) =>
  stopWith(
    listen<GameKeyFailure>('openai-key-failure', ({ payload }) => {
      handler(payload);
    }),
  );

export const onWidgetSwitches = (handler: (switches: WidgetSwitches) => void): (() => void) =>
  stopWith(
    listen<WidgetSwitches>('widget-switches', ({ payload }) => {
      handler(payload);
    }),
  );
