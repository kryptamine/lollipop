import { SwitchRow } from './SwitchRow';
import type { SwitchControls } from './use-switches';

type GeneralTabProps = {
  controls: SwitchControls;
};

export const GeneralTab = ({
  controls: { switches, onLaunchAtLoginChange, onRecapWindowChange },
}: GeneralTabProps) => (
  <>
    <div class="caption">Startup</div>
    <div class="group">
      <SwitchRow
        label="Launch at login"
        isOn={switches.launchAtLogin}
        onChange={onLaunchAtLoginChange}
      />
    </div>
    <div class="caption">After a game</div>
    <div class="group">
      <SwitchRow
        label="Recap after a game"
        subtitle="Opens over the client once the game is over"
        isOn={switches.recap.window}
        onChange={onRecapWindowChange}
      />
    </div>
  </>
);
