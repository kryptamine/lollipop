import { BuildIcon } from './BuildIcon';
import type { TimingHintState } from './bridge';

export const TimingHint = ({ objective, countdown, elixir }: TimingHintState) => (
  <div class="timing-hint">
    {objective} {countdown}
    {elixir !== null && <BuildIcon src={elixir.icon} isNext />}
  </div>
);
