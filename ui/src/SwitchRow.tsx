import { Switch } from './Switch';

type SwitchRowProps = {
  label: string;
  isOn: boolean;
  isDisabled?: boolean;
  // Says why a disabled switch is disabled.
  subtitle?: string | undefined;
  onChange: (isOn: boolean) => void;
};

export const SwitchRow = ({
  label,
  isOn,
  isDisabled = false,
  subtitle,
  onChange,
}: SwitchRowProps) => (
  <div class="row">
    <div class="label">
      <span>{label}</span>
      {subtitle !== undefined && <span class="subtitle">{subtitle}</span>}
    </div>
    <Switch label={label} isOn={isOn} isDisabled={isDisabled} onChange={onChange} />
  </div>
);
