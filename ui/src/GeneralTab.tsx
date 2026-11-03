import { SwitchRow } from './SwitchRow';

type GeneralTabProps = {
  launchAtLogin: boolean;
  onLaunchAtLoginChange: (isOn: boolean) => void;
};

export const GeneralTab = ({ launchAtLogin, onLaunchAtLoginChange }: GeneralTabProps) => (
  <>
    <div class="caption">Startup</div>
    <div class="group">
      <SwitchRow label="Launch at login" isOn={launchAtLogin} onChange={onLaunchAtLoginChange} />
    </div>
  </>
);
