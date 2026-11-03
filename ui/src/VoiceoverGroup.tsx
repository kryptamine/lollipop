import { SwitchRow } from './SwitchRow';

type VoiceoverGroupProps = {
  isWidgetOn: boolean;
  // `undefined` while the app has not said yet.
  isVoiceInstalled: boolean | undefined;
  isOn: boolean;
  onChange: (isOn: boolean) => void;
};

const disabledReason = (isWidgetOn: boolean, isVoiceInstalled: boolean | undefined) => {
  if (!isWidgetOn) {
    return 'Needs the Advice Widget';
  }

  return isVoiceInstalled === false ? 'No voice installed for this language' : undefined;
};

export const VoiceoverGroup = ({
  isWidgetOn,
  isVoiceInstalled,
  isOn,
  onChange,
}: VoiceoverGroupProps) => {
  const reason = disabledReason(isWidgetOn, isVoiceInstalled);
  const isDisabled = isVoiceInstalled !== true || reason !== undefined;

  return (
    <>
      <div class="caption">Voiceover</div>
      <div class={reason === undefined ? 'group' : 'group dim'}>
        <SwitchRow
          label="Read Advice aloud"
          isOn={isOn}
          isDisabled={isDisabled}
          subtitle={reason}
          onChange={onChange}
        />
      </div>
      <div class="footer">
        Better voices download in System Settings → Accessibility → Spoken Content.
      </div>
    </>
  );
};
