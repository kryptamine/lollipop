// The only module that may import Tauri; components stay unaware of it.
import { listen } from "@tauri-apps/api/event";

export interface Snapshot {
  cs: number;
}

export function onSnapshot(handler: (snapshot: Snapshot) => void): () => void {
  const unlisten = listen<Snapshot>("snapshot", ({ payload }) => {
    handler(payload);
  });
  return () => {
    void unlisten.then((stop) => {
      stop();
    });
  };
}
