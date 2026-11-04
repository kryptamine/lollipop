import { BuildIcon } from './BuildIcon';
import type { BuildItem } from './bridge';

type BuildWidgetProps = {
  // In build order, the Next item first; empty with no Build.
  build: BuildItem[];
  // Why a model just replaced the Next item.
  change: string | undefined;
};

export const BuildWidget = ({ build, change }: BuildWidgetProps) =>
  build.length > 0 && (
    <div id="build-widget">
      <div class="build-items">
        {build.map(({ item, icon }, index) => (
          <BuildIcon key={item} src={icon} isNext={index === 0} />
        ))}
      </div>
      {change !== undefined && <p class="build-change">{change}</p>}
    </div>
  );
