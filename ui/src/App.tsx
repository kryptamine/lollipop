import { useEffect, useState } from "preact/hooks";

import { onSnapshot } from "./bridge";

export function App() {
  const [cs, setCs] = useState(0);

  useEffect(
    () =>
      onSnapshot((snapshot) => {
        setCs(snapshot.cs);
      }),
    [],
  );

  return <div id="pill">CS {cs}</div>;
}
