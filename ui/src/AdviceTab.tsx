import type { ComponentChildren } from 'preact';

import { KeyField } from './KeyField';
import type { KeyFailure } from './bridge';
import type { OpenAiKeyControls } from './use-open-ai-key';

const FAILURE_TEXT: Record<KeyFailure, string> = {
  rejected: 'OpenAI rejected the key',
  unreachable: 'Could not reach OpenAI, try again',
  notKept: 'The Keychain did not keep the key, try again',
};

type AdviceTabProps = {
  openAiKey: OpenAiKeyControls;
  isWidgetOn: boolean;
  onUsageOpen: () => void;
  onOverlayOpen: () => void;
  // The groups under the key, greyed by their own components while the Widget is off.
  children: ComponentChildren;
};

export const AdviceTab = ({
  openAiKey: { saved, isChecking, failure, onKeyEntered, onRemove },
  isWidgetOn,
  onUsageOpen,
  onOverlayOpen,
  children,
}: AdviceTabProps) => (
  <>
    <div class="caption">OpenAI</div>
    <div class="group">
      <div class="row">
        <div class="label">
          <span>API key</span>
          {/* The figure is measured on the Advice model: update it when the model changes. */}
          <button type="button" class="link subtitle" onClick={onUsageOpen}>
            About $0.005 per game, billed to your OpenAI account
          </button>
        </div>
        <KeyField
          saved={saved}
          isChecking={isChecking}
          onKeyEntered={onKeyEntered}
          onRemove={onRemove}
        />
      </div>
      {failure !== undefined && (
        <div class="note" role="alert">
          {FAILURE_TEXT[failure]}
        </div>
      )}
    </div>
    {!isWidgetOn && (
      <div class="banner">
        The Advice Widget is off.{' '}
        <button type="button" class="link" onClick={onOverlayOpen}>
          Turn it on in Overlay
        </button>
      </div>
    )}
    {children}
  </>
);
