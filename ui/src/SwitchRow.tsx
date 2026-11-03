import { Switch } from './Switch';

type SwitchRowProps = {
  label: string;
  isOn: boolean;
  onChange: (isOn: boolean) => void;
};

export const SwitchRow = ({ label, isOn, onChange }: SwitchRowProps) => (
  <div class="row">
    <span>{label}</span>
    <Switch label={label} isOn={isOn} onChange={onChange} />
  </div>
);
