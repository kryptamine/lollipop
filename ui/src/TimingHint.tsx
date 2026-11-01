import type { TimingHintState } from './bridge';

export const TimingHint = ({ objective, countdown }: TimingHintState) => (
  <div class="timing-hint">
    {objective} {countdown}
  </div>
);
