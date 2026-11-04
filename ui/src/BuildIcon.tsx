import { useState } from 'preact/hooks';

type BuildIconProps = {
  src: string | null;
  isNext: boolean;
};

// An icon missing or failing to load keeps its slot empty, so the order still reads.
export const BuildIcon = ({ src, isNext }: BuildIconProps) => {
  const [hasFailed, setHasFailed] = useState(false);
  const className = isNext ? 'build-item next' : 'build-item';

  if (src === null || hasFailed) {
    return <div class={className} />;
  }

  return (
    <img
      class={className}
      src={src}
      alt=""
      width={32}
      height={32}
      onError={() => {
        setHasFailed(true);
      }}
    />
  );
};
