import { useEffect, useState } from 'preact/hooks';

import { AdviceTab } from './AdviceTab';
import { GeneralTab } from './GeneralTab';
import { OverlayTab } from './OverlayTab';
import { useOpenAiKey } from './use-open-ai-key';
import { SettingsToolbar, type SettingsTab } from './SettingsToolbar';
import {
  openOpenAiUsage,
  readSwitches,
  setLaunchAtLogin,
  setWidget,
  type Switches,
  type WidgetId,
} from './bridge';

const handleUsageOpen = () => {
  void openOpenAiUsage();
};

export const SettingsWindow = () => {
  const [tab, setTab] = useState<SettingsTab>('general');
  const [switches, setSwitches] = useState<Switches>();
  const openAiKey = useOpenAiKey();

  const show = async (kept: Promise<Switches>) => {
    setSwitches(await kept);
  };

  useEffect(() => {
    void show(readSwitches());
  }, []);

  const handleWidgetChange = (id: WidgetId, isOn: boolean) => {
    void show(setWidget(id, isOn));
  };

  const handleLaunchAtLoginChange = (isOn: boolean) => {
    void show(setLaunchAtLogin(isOn));
  };

  return (
    <>
      <SettingsToolbar shown={tab} onSelect={setTab} />
      <main>
        {switches && tab === 'general' && (
          <GeneralTab
            launchAtLogin={switches.launchAtLogin}
            onLaunchAtLoginChange={handleLaunchAtLoginChange}
          />
        )}
        {switches && tab === 'overlay' && (
          <OverlayTab widgets={switches.widgets} onWidgetChange={handleWidgetChange} />
        )}
        {openAiKey && tab === 'advice' && (
          <AdviceTab openAiKey={openAiKey} onUsageOpen={handleUsageOpen} />
        )}
      </main>
    </>
  );
};
