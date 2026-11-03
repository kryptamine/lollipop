import type { TargetedEvent } from 'preact';

import type { AdviceLanguage } from './bridge';

type LanguageOption = {
  value: AdviceLanguage;
  label: string;
};

const OPTIONS: LanguageOption[] = [
  {
    value: 'system',
    label: 'System default',
  },
  {
    value: 'en',
    label: 'English',
  },
  {
    value: 'ru',
    label: 'Русский',
  },
];

type LanguageRowProps = {
  language: AdviceLanguage;
  isDisabled: boolean;
  onChange: (language: AdviceLanguage) => void;
};

export const LanguageRow = ({ language, isDisabled, onChange }: LanguageRowProps) => {
  const handleChange = (event: TargetedEvent<HTMLSelectElement>) => {
    const chosen = OPTIONS.find(({ value }) => value === event.currentTarget.value);

    if (chosen !== undefined) {
      onChange(chosen.value);
    }
  };

  return (
    <div class="row">
      <span>Language</span>
      <select aria-label="Language" value={language} disabled={isDisabled} onChange={handleChange}>
        {OPTIONS.map(({ value, label }) => (
          <option key={value} value={value}>
            {label}
          </option>
        ))}
      </select>
    </div>
  );
};
