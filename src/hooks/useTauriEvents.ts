import { useEffect, useRef } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import type { NetworkEvent } from "../types";
import { useNetworkStore } from "../stores";

/**
 * Hook to listen to Tauri events from the Rust backend.
 * Should be called once at the app root level.
 */
export function useTauriEvents() {
  const unlistenersRef = useRef<UnlistenFn[]>([]);
  const { refreshPeers, refreshStats } = useNetworkStore();

  useEffect(() => {
    async function setupListeners() {
      // Listen to network events
      const unlistenNetwork = await listen<NetworkEvent>(
        "harbor:network",
        (event) => {
          console.log("[TauriEvent] harbor:network:", event.payload);
          handleNetworkEvent(event.payload);
        }
      );
      unlistenersRef.current.push(unlistenNetwork);

      // Future: Listen to message events
      // const unlistenMessage = await listen<MessageEvent>(
      //   "harbor:message",
      //   (event) => handleMessageEvent(event.payload)
      // );
      // unlistenersRef.current.push(unlistenMessage);
    }

    function handleNetworkEvent(event: NetworkEvent) {
      switch (event.type) {
        case "peer_connected":
          console.log(`[Network] Peer connected: ${event.peerId}`);
          // Refresh the full peer list to get updated info
          refreshPeers();
          refreshStats();
          break;

        case "peer_disconnected":
          console.log(`[Network] Peer disconnected: ${event.peerId}`);
          refreshPeers();
          refreshStats();
          break;

        case "peer_discovered":
          console.log(`[Network] Peer discovered: ${event.peerId}`);
          refreshPeers();
          break;

        case "peer_expired":
          console.log(`[Network] Peer expired: ${event.peerId}`);
          refreshPeers();
          break;

        case "message_received":
          console.log(
            `[Network] Message received from ${event.peerId} via ${event.protocol}`
          );
          // TODO: Route to messaging store when implemented
          break;

        case "listening_on":
          console.log(`[Network] Listening on: ${event.address}`);
          break;

        case "external_address_discovered":
          console.log(`[Network] External address: ${event.address}`);
          break;

        case "status_changed":
          console.log(`[Network] Status changed: ${event.status}`);
          break;
      }
    }

    setupListeners();

    // Cleanup on unmount
    return () => {
      unlistenersRef.current.forEach((unlisten) => unlisten());
      unlistenersRef.current = [];
    };
  }, [refreshPeers, refreshStats]);
}
