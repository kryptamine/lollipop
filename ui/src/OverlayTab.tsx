import { SwitchRow } from './SwitchRow';
import type { WidgetId, WidgetSwitches } from './bridge';

type OverlayTabProps = {
  widgets: WidgetSwitches;
  hasKey: boolean;
  onWidgetChange: (id: WidgetId, isOn: boolean) => void;
};

export const OverlayTab = ({
  widgets: { markerStrip, timingHints, adviceWidget },
  hasKey,
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
      <SwitchRow
        label="Advice Widget"
        isOn={adviceWidget && hasKey}
        isDisabled={!hasKey}
        subtitle={hasKey ? undefined : 'Add an OpenAI key in the Advice tab'}
        onChange={(isOn) => {
          onWidgetChange('adviceWidget', isOn);
        }}
      />
    </div>
  </>
);
