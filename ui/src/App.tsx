import { useEffect, useState } from 'preact/hooks';

import { onSnapshot } from './bridge';

export const App = () => {
  const [cs, setCs] = useState(0);

  useEffect(
    () =>
      onSnapshot(({ cs: nextCs }) => {
        setCs(nextCs);
      }),
    [],
  );

  return <div id="pill">CS {cs}</div>;
};
