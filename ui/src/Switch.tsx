type SwitchProps = {
  label: string;
  isOn: boolean;
  onChange: (isOn: boolean) => void;
};

export const Switch = ({ label, isOn, onChange }: SwitchProps) => (
  <button
    type="button"
    role="switch"
    class="switch"
    aria-label={label}
    aria-checked={isOn}
    onClick={() => {
      onChange(!isOn);
    }}
  >
    <span />
  </button>
);
