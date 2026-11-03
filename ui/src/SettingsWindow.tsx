import { useState } from 'preact/hooks';

import { ShownTab } from './ShownTab';
import { openOpenAiUsage } from './bridge';
import { useSwitches } from './use-switches';
import { useOpenAiKey } from './use-open-ai-key';
import { SettingsToolbar, type SettingsTab } from './SettingsToolbar';

const handleUsageOpen = () => {
  void openOpenAiUsage();
};

export const SettingsWindow = () => {
  const [tab, setTab] = useState<SettingsTab>('general');
  const controls = useSwitches();
  const openAiKey = useOpenAiKey();

  const handleOverlayOpen = () => {
    setTab('overlay');
  };

  return (
    <>
      <SettingsToolbar shown={tab} onSelect={setTab} />
      <main>
        {controls && openAiKey && (
          <ShownTab
            tab={tab}
            controls={controls}
            openAiKey={openAiKey}
            onUsageOpen={handleUsageOpen}
            onOverlayOpen={handleOverlayOpen}
          />
        )}
      </main>
    </>
  );
};
