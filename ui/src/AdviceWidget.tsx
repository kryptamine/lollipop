import { AdviceIcon } from './AdviceIcon';
import type { ShownAdvice, TriggerId } from './bridge';

const KICKER: Record<TriggerId, string> = {
  loadingScreen: 'Lane plan',
  backToBase: 'Back to base',
  turningPoint: 'Turning point',
  death: 'Death',
};

type AdviceWidgetProps = {
  // Nothing shows while no Advice is.
  advice: ShownAdvice | undefined;
};

export const AdviceWidget = ({ advice }: AdviceWidgetProps) =>
  advice && (
    <div id="advice-widget">
      <div class="advice-kicker">{KICKER[advice.trigger]}</div>
      <div class="advice-body">
        {/* Keyed by its URL, so a failed icon does not hide the next Advice's. */}
        {advice.icon !== null && <AdviceIcon key={advice.icon} src={advice.icon} />}
        <div>{advice.text}</div>
      </div>
      <div class="advice-time">
        <div style={{ width: `${advice.timeLeft * 100}%` }} />
      </div>
    </div>
  );
