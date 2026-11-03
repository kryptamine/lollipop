import { useEffect, useState } from 'preact/hooks';

import {
  onWidgetSwitches,
  readSwitches,
  setAdviceLanguage,
  setLaunchAtLogin,
  setTrigger,
  setVoiceover,
  setWidget,
  type AdviceLanguage,
  type Switches,
  type TriggerId,
  type WidgetId,
} from './bridge';

export type SwitchControls = {
  switches: Switches;
  onWidgetChange: (id: WidgetId, isOn: boolean) => void;
  onTriggerChange: (id: TriggerId, isOn: boolean) => void;
  onLanguageChange: (language: AdviceLanguage) => void;
  onLaunchAtLoginChange: (isOn: boolean) => void;
  onVoiceoverChange: (isOn: boolean) => void;
};

// The switches as the app kept them, `undefined` until it has answered.
export const useSwitches = (): SwitchControls | undefined => {
  const [switches, setSwitches] = useState<Switches>();

  const show = async (kept: Promise<Switches>) => {
    setSwitches(await kept);
  };

  useEffect(() => {
    void show(readSwitches());
  }, []);

  // Removing the key turns the Advice Widget off from the app's side.
  useEffect(
    () =>
      onWidgetSwitches((widgets) => {
        setSwitches(
          (shown) =>
            shown && {
              ...shown,
              widgets,
            },
        );
      }),
    [],
  );

  if (switches === undefined) {
    return undefined;
  }

  return {
    switches,
    onWidgetChange: (id, isOn) => {
      void show(setWidget(id, isOn));
    },
    onTriggerChange: (id, isOn) => {
      void show(setTrigger(id, isOn));
    },
    onLanguageChange: (language) => {
      void show(setAdviceLanguage(language));
    },
    onLaunchAtLoginChange: (isOn) => {
      void show(setLaunchAtLogin(isOn));
    },
    onVoiceoverChange: (isOn) => {
      void show(setVoiceover(isOn));
    },
  };
};
