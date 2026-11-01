import type { MarkerState } from './bridge';

type MarkerProps = {
  state: MarkerState | null;
};

// The slot is always there, so no change of state can shift the strip's layout.
export const Marker = ({ state }: MarkerProps) => (
  <span class={`marker marker-${state ?? 'none'}`} />
);
