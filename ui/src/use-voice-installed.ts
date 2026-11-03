import { useEffect, useState } from 'preact/hooks';

import { isVoiceInstalled, type AdviceLanguage } from './bridge';

// Whether a voice reads `language`, `undefined` until the app has answered for it.
export const useVoiceInstalled = (language: AdviceLanguage): boolean | undefined => {
  type VoiceAnswer = {
    language: AdviceLanguage;
    isInstalled: boolean;
  };

  const [answer, setAnswer] = useState<VoiceAnswer>();

  useEffect(() => {
    let isCurrent = true;
    const read = async () => {
      const isInstalled = await isVoiceInstalled();

      if (isCurrent) {
        setAnswer({
          language,
          isInstalled,
        });
      }
    };

    void read();

    return () => {
      isCurrent = false;
    };
  }, [language]);

  return answer?.language === language ? answer.isInstalled : undefined;
};
