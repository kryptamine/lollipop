import { AdviceTab } from './AdviceTab';
import { GeneralTab } from './GeneralTab';
import { OverlayTab } from './OverlayTab';
import { AdviceGroup } from './AdviceGroup';
import type { SettingsTab } from './SettingsToolbar';
import type { SwitchControls } from './use-switches';
import type { OpenAiKeyControls } from './use-open-ai-key';

type ShownTabProps = {
  tab: SettingsTab;
  controls: SwitchControls;
  openAiKey: OpenAiKeyControls;
  onUsageOpen: () => void;
  onOverlayOpen: () => void;
};

export const ShownTab = ({
  tab,
  controls: { switches, onWidgetChange, onTriggerChange, onLanguageChange, onLaunchAtLoginChange },
  openAiKey,
  onUsageOpen,
  onOverlayOpen,
}: ShownTabProps) => {
  if (tab === 'general') {
    return (
      <GeneralTab
        launchAtLogin={switches.launchAtLogin}
        onLaunchAtLoginChange={onLaunchAtLoginChange}
      />
    );
  }

  const hasKey = openAiKey.saved !== undefined;

  if (tab === 'overlay') {
    return (
      <OverlayTab widgets={switches.widgets} hasKey={hasKey} onWidgetChange={onWidgetChange} />
    );
  }

  // Without a key the Widget shows nothing, whatever its stored switch says.
  const isWidgetOn = hasKey && switches.widgets.adviceWidget;

  return (
    <AdviceTab
      openAiKey={openAiKey}
      isWidgetOn={isWidgetOn}
      onUsageOpen={onUsageOpen}
      onOverlayOpen={onOverlayOpen}
    >
      <AdviceGroup
        isWidgetOn={isWidgetOn}
        language={switches.adviceLanguage}
        triggers={switches.triggers}
        onLanguageChange={onLanguageChange}
        onTriggerChange={onTriggerChange}
      />
    </AdviceTab>
  );
};
