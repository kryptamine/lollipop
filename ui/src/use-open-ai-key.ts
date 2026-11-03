import { useEffect, useState } from 'preact/hooks';

import {
  onOpenAiKeyFailure,
  readOpenAiKey,
  readOpenAiKeyFailure,
  removeOpenAiKey,
  saveOpenAiKey,
  type KeyFailure,
} from './bridge';

export type OpenAiKeyControls = {
  saved: string | undefined;
  isChecking: boolean;
  failure: KeyFailure | undefined;
  onKeyEntered: (key: string) => void;
  onRemove: () => void;
};

// The key as the Advice tab draws it, `undefined` until the app has answered.
export const useOpenAiKey = (): OpenAiKeyControls | undefined => {
  const [isRead, setIsRead] = useState(false);
  const [saved, setSaved] = useState<string>();
  const [isChecking, setIsChecking] = useState(false);
  const [failure, setFailure] = useState<KeyFailure>();

  useEffect(() => {
    const read = async () => {
      setSaved(await readOpenAiKey());
      setFailure(await readOpenAiKeyFailure());
      setIsRead(true);
    };

    void read();
  }, []);

  // A key OpenAI refuses in a game is marked while the tab is open.
  useEffect(() => onOpenAiKeyFailure(setFailure), []);

  if (!isRead) {
    return undefined;
  }

  const check = async (key: string) => {
    setIsChecking(true);
    setFailure(undefined);
    try {
      const answer = await saveOpenAiKey(key);

      if (answer.outcome === 'saved') {
        setSaved(answer.masked);
      } else {
        setFailure(answer.outcome);
      }
    } finally {
      setIsChecking(false);
    }
  };

  const remove = async () => {
    setFailure(undefined);
    setSaved(await removeOpenAiKey());
  };

  return {
    saved,
    isChecking,
    failure,
    onKeyEntered: (key) => {
      void check(key);
    },
    onRemove: () => {
      void remove();
    },
  };
};
