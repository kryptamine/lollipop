import { SwitchRow } from './SwitchRow';
import { LanguageRow } from './LanguageRow';
import type { AdviceLanguage, TriggerId, TriggerSwitches } from './bridge';

type AdviceGroupProps = {
  isWidgetOn: boolean;
  language: AdviceLanguage;
  triggers: TriggerSwitches;
  onLanguageChange: (language: AdviceLanguage) => void;
  onTriggerChange: (id: TriggerId, isOn: boolean) => void;
};

// Greyed but visible with the Advice Widget off, which is the only Advice on/off.
export const AdviceGroup = ({
  isWidgetOn,
  language,
  triggers: { loadingScreen, death },
  onLanguageChange,
  onTriggerChange,
}: AdviceGroupProps) => (
  <>
    <div class="caption">Advice</div>
    <div class={isWidgetOn ? 'group' : 'group dim'}>
      <LanguageRow language={language} isDisabled={!isWidgetOn} onChange={onLanguageChange} />
      <SwitchRow
        label="Loading screen"
        isOn={loadingScreen}
        isDisabled={!isWidgetOn}
        onChange={(isOn) => {
          onTriggerChange('loadingScreen', isOn);
        }}
      />
      <SwitchRow
        label="Death"
        isOn={death}
        isDisabled={!isWidgetOn}
        onChange={(isOn) => {
          onTriggerChange('death', isOn);
        }}
      />
    </div>
  </>
);
