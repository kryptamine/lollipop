type SwitchProps = {
  label: string;
  isOn: boolean;
  isDisabled?: boolean;
  onChange: (isOn: boolean) => void;
};

export const Switch = ({ label, isOn, isDisabled = false, onChange }: SwitchProps) => (
  <button
    type="button"
    role="switch"
    class="switch"
    aria-label={label}
    aria-checked={isOn}
    disabled={isDisabled}
    onClick={() => {
      onChange(!isOn);
    }}
  >
    <span />
  </button>
);
