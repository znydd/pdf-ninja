import { useEffect, useState, useCallback } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

/**
 * Custom hook that listens for "image-received" Tauri events
 * and maintains a list of received image paths.
 */
export function useImageReceiver() {
  const [images, setImages] = useState<string[]>([]);

  useEffect(() => {
    let cancelled = false;
    let unlisten: UnlistenFn | undefined;

    const setup = async () => {
      const fn = await listen<string>("image-received", (event) => {
        if (!cancelled) {
          setImages((prev) => [...prev, event.payload]);
        }
      });

      // If the effect was already cleaned up before the promise resolved,
      // immediately unsubscribe. Otherwise, store the unsubscribe function.
      if (cancelled) {
        fn();
      } else {
        unlisten = fn;
      }
    };

    setup();

    return () => {
      cancelled = true;
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  const clearImages = useCallback(() => {
    setImages([]);
  }, []);

  return { images, clearImages };
}
