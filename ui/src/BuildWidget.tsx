import { BuildIcon } from './BuildIcon';
import type { BuildItem } from './bridge';

type BuildWidgetProps = {
  // In build order, the Next item first; empty with no Build.
  build: BuildItem[];
};

export const BuildWidget = ({ build }: BuildWidgetProps) =>
  build.length > 0 && (
    <div id="build-widget">
      {build.map(({ item, icon }, index) => (
        <BuildIcon key={item} src={icon} isNext={index === 0} />
      ))}
    </div>
  );
