import { TimingHint } from './TimingHint';
import type { TimingHintState } from './bridge';

type TimingHintBannerProps = {
  hints: TimingHintState[];
};

export const TimingHintBanner = ({ hints }: TimingHintBannerProps) => (
  <div id="timing-hint-banner">
    {hints.map(({ objective, countdown }) => (
      <TimingHint key={objective} objective={objective} countdown={countdown} />
    ))}
  </div>
);
