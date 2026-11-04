import { SwapRow } from './SwapRow';
import { BuildIcon } from './BuildIcon';
import type { BuildItem, ShownSwap } from './bridge';

type BuildWidgetProps = {
  // In build order, the Next item first; empty with no Build.
  build: BuildItem[];
  // Why a model just replaced the Next item.
  change: string | undefined;
  // Only while the inventory is full, when the Build is empty.
  swap: ShownSwap | undefined;
};

export const BuildWidget = ({ build, change, swap }: BuildWidgetProps) =>
  (build.length > 0 || swap !== undefined) && (
    <div id="build-widget">
      {build.length > 0 && (
        <div class="build-items">
          {build.map(({ item, icon }, index) => (
            <BuildIcon key={item} src={icon} isNext={index === 0} />
          ))}
        </div>
      )}
      {swap !== undefined && <SwapRow swap={swap} />}
      {change !== undefined && <p class="build-change">{change}</p>}
    </div>
  );
