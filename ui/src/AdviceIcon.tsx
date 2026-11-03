import { useState } from 'preact/hooks';

type AdviceIconProps = {
  src: string;
};

// The Advice names the item in its text, so an icon that fails to load is simply left out.
export const AdviceIcon = ({ src }: AdviceIconProps) => {
  const [hasFailed, setHasFailed] = useState(false);

  return (
    !hasFailed && (
      <img
        class="advice-icon"
        src={src}
        alt=""
        width={32}
        height={32}
        onError={() => {
          setHasFailed(true);
        }}
      />
    )
  );
};
