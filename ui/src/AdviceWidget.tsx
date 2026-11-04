import type { ShownAdvice, TriggerId } from './bridge';

const KICKER: Record<TriggerId, string> = {
  loadingScreen: 'Lane plan',
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
      <div>{advice.text}</div>
      <div class="advice-time">
        <div style={{ width: `${advice.timeLeft * 100}%` }} />
      </div>
    </div>
  );
