import type { TargetedClipboardEvent, TargetedKeyboardEvent } from 'preact';

type KeyFieldProps = {
  saved: string | undefined;
  isChecking: boolean;
  onKeyEntered: (key: string) => void;
  onRemove: () => void;
};

// A paste or Enter checks the key at once: there is no Save button.
export const KeyField = ({ saved, isChecking, onKeyEntered, onRemove }: KeyFieldProps) => {
  const enter = (input: HTMLInputElement, key: string) => {
    input.value = '';
    if (key !== '') {
      onKeyEntered(key);
    }
  };

  const handlePaste = (event: TargetedClipboardEvent<HTMLInputElement>) => {
    event.preventDefault();
    enter(event.currentTarget, event.clipboardData?.getData('text').trim() ?? '');
  };

  const handleKeyDown = (event: TargetedKeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') {
      enter(event.currentTarget, event.currentTarget.value.trim());
    }
  };

  return (
    <div class="key-field">
      {isChecking && <span class="spinner" role="status" aria-label="Checking the key" />}
      <input
        type="password"
        aria-label="API key"
        placeholder={saved ?? 'Paste your key'}
        class={saved === undefined ? undefined : 'saved'}
        disabled={isChecking}
        autocomplete="off"
        spellcheck={false}
        onPaste={handlePaste}
        onKeyDown={handleKeyDown}
      />
      {saved !== undefined && (
        <button type="button" class="link" onClick={onRemove}>
          Remove key
        </button>
      )}
    </div>
  );
};
