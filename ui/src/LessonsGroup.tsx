import { SwitchRow } from './SwitchRow';

type LessonsGroupProps = {
  isWidgetOn: boolean;
  isOn: boolean;
  onChange: (isOn: boolean) => void;
};

// Lessons are Advice, so the Advice Widget's switch turns them off too.
export const LessonsGroup = ({ isWidgetOn, isOn, onChange }: LessonsGroupProps) => (
  <>
    <div class="caption">Recap</div>
    <div class={isWidgetOn ? 'group' : 'group dim'}>
      <SwitchRow
        label="Lessons after a game"
        subtitle={isWidgetOn ? undefined : 'Needs the Advice Widget'}
        isOn={isOn}
        isDisabled={!isWidgetOn}
        onChange={onChange}
      />
    </div>
  </>
);
