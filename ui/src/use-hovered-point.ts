import { useState } from 'preact/hooks';

import { LEFT, RIGHT, WIDTH, type ChartPoint } from './chart-geometry';

export type HoveredPoint = {
  hovered: ChartPoint | undefined;
  handlePointerMove: (event: PointerEvent) => void;
  handlePointerLeave: () => void;
};

const nearestTo = (points: ChartPoint[], minute: number): ChartPoint | undefined => {
  let nearest: ChartPoint | undefined;

  for (const point of points) {
    if (
      nearest === undefined ||
      Math.abs(point.minute - minute) < Math.abs(nearest.minute - minute)
    ) {
      nearest = point;
    }
  }

  return nearest;
};

// The point under the pointer, by the minute nearest it.
export const useHoveredPoint = (points: ChartPoint[], lastMinute: number): HoveredPoint => {
  const [hovered, setHovered] = useState<ChartPoint>();

  const handlePointerMove = (event: PointerEvent) => {
    const box = (event.currentTarget as SVGSVGElement).getBoundingClientRect();
    const x = ((event.clientX - box.left) / box.width) * WIDTH;
    const minute = ((x - LEFT) / (WIDTH - LEFT - RIGHT)) * Math.max(1, lastMinute);

    setHovered(nearestTo(points, minute));
  };

  const handlePointerLeave = () => {
    setHovered(undefined);
  };

  return {
    hovered,
    handlePointerMove,
    handlePointerLeave,
  };
};
