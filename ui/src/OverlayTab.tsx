import { SwitchRow } from './SwitchRow';
import type { WidgetId, WidgetSwitches } from './bridge';

type OverlayTabProps = {
  widgets: WidgetSwitches;
  onWidgetChange: (id: WidgetId, isOn: boolean) => void;
};

export const OverlayTab = ({
  widgets: { markerStrip, timingHints },
  onWidgetChange,
}: OverlayTabProps) => (
  <>
    <div class="caption">Widgets</div>
    <div class="group">
      <SwitchRow
        label="Marker strip"
        isOn={markerStrip}
        onChange={(isOn) => {
          onWidgetChange('markerStrip', isOn);
        }}
      />
      <SwitchRow
        label="Timing hints"
        isOn={timingHints}
        onChange={(isOn) => {
          onWidgetChange('timingHints', isOn);
        }}
      />
    </div>
  </>
);
