import { BuildIcon } from './BuildIcon';
import type { ShownSwap } from './bridge';

type SwapRowProps = {
  swap: ShownSwap;
};

export const SwapRow = ({ swap: { sell, buy, reason } }: SwapRowProps) => (
  <>
    <div class="build-items">
      <BuildIcon src={sell.icon} isNext={false} />
      <span class="swap-arrow">→</span>
      <BuildIcon src={buy.icon} isNext />
    </div>
    <p class="build-change">{reason}</p>
  </>
);
