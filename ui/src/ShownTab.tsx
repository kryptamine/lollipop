import { AdviceTab } from './AdviceTab';
import { GeneralTab } from './GeneralTab';
import { OverlayTab } from './OverlayTab';
import { AdviceGroup } from './AdviceGroup';
import { LessonsGroup } from './LessonsGroup';
import { VoiceoverGroup } from './VoiceoverGroup';
import type { SettingsTab } from './SettingsToolbar';
import type { SwitchControls } from './use-switches';
import { useVoiceInstalled } from './use-voice-installed';
import type { OpenAiKeyControls } from './use-open-ai-key';

type ShownTabProps = {
  tab: SettingsTab;
  controls: SwitchControls;
  openAiKey: OpenAiKeyControls;
  onUsageOpen: () => void;
  onOverlayOpen: () => void;
};

// `controls` stays whole: the General tab is handed it whole.
export const ShownTab = ({
  tab,
  controls,
  openAiKey,
  onUsageOpen,
  onOverlayOpen,
}: ShownTabProps) => {
  const { switches } = controls;
  const isVoiceInstalled = useVoiceInstalled(switches.adviceLanguage);

  if (tab === 'general') {
    return <GeneralTab controls={controls} />;
  }

  const hasKey = openAiKey.saved !== undefined;

  if (tab === 'overlay') {
    return (
      <OverlayTab
        widgets={switches.widgets}
        hasKey={hasKey}
        onWidgetChange={controls.onWidgetChange}
      />
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
        onLanguageChange={controls.onLanguageChange}
        onTriggerChange={controls.onTriggerChange}
      />
      <LessonsGroup
        isWidgetOn={isWidgetOn}
        isOn={switches.recap.lessons}
        onChange={controls.onLessonsChange}
      />
      <VoiceoverGroup
        isWidgetOn={isWidgetOn}
        isVoiceInstalled={isVoiceInstalled}
        isOn={switches.voiceover}
        onChange={controls.onVoiceoverChange}
      />
    </AdviceTab>
  );
};
