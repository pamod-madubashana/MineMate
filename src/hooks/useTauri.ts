import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useRef, useState, useCallback, useMemo } from "react";

// Bot types
export interface BotStatus {
  connected: boolean;
  server: string;
  username: string;
  health: number;
  food: number;
  x: number;
  y: number;
  z: number;
  uptime_seconds: number;
}

export interface BotEvent {
  type: string;
  data: any;
}

// Config types
export interface AppConfig {
  server: {
    address: string;
    port: number;
    auto_reconnect: boolean;
  };
  bot: {
    username: string;
    permission_mode: string;
  };
  ai: {
    api_key: string;
    model: string;
    temperature: number;
    max_tokens: number;
  };
  automation: {
    auto_sleep: boolean;
    auto_eat: boolean;
    auto_reconnect: boolean;
    welcome_messages: boolean;
    starter_kit_on_respawn: boolean;
  };
  starter_kit: Array<{
    item: string;
    count: number;
  }>;
}

// Memory types
export interface Player {
  id: number;
  name: string;
  first_seen: string;
  last_seen: string;
  preferences: string | null;
}

export interface Location {
  id: number;
  name: string;
  x: number;
  y: number;
  z: number;
  dimension: string;
  description: string | null;
}

export interface HistoryEntry {
  id: number;
  timestamp: string;
  event_type: string;
  player: string | null;
  details: string | null;
}

export interface Blueprint {
  id: number;
  name: string;
  data: string;
  author: string | null;
  created: string;
}

// Bot commands
export const useBot = () => {
  const startBot = useCallback(async (server: string, username: string) => {
    return invoke("start_bot", { server, username });
  }, []);

  const stopBot = useCallback(async () => {
    return invoke("stop_bot");
  }, []);

  const getBotStatus = useCallback(async (): Promise<BotStatus> => {
    return invoke("get_bot_status");
  }, []);

  const sendChat = useCallback(async (message: string) => {
    return invoke("send_chat", { message });
  }, []);

  const getConnectionStatus = useCallback(async (): Promise<boolean> => {
    return invoke("get_connection_status");
  }, []);

  return useMemo(() => ({ startBot, stopBot, getBotStatus, sendChat, getConnectionStatus }), [startBot, stopBot, getBotStatus, sendChat, getConnectionStatus]);
};

// Config commands
export const useConfig = () => {
  const getConfig = useCallback(async (): Promise<AppConfig> => {
    return invoke("get_config");
  }, []);

  const saveConfig = useCallback(async (config: AppConfig) => {
    return invoke("save_config", { config });
  }, []);

  return useMemo(() => ({ getConfig, saveConfig }), [getConfig, saveConfig]);
};

// Memory commands
export const useMemory = () => {
  const listPlayers = useCallback(async (): Promise<Player[]> => {
    return invoke("list_players");
  }, []);

  const savePlayer = useCallback(async (name: string): Promise<Player> => {
    return invoke("save_player", { name });
  }, []);

  const listLocations = useCallback(async (): Promise<Location[]> => {
    return invoke("list_locations");
  }, []);

  const saveLocation = useCallback(async (name: string, x: number, y: number, z: number, dimension: string, description: string): Promise<Location> => {
    return invoke("save_location", { name, x, y, z, dimension, description });
  }, []);

  const listBlueprints = useCallback(async (): Promise<Blueprint[]> => {
    return invoke("list_blueprints");
  }, []);

  const saveBlueprint = useCallback(async (name: string, data: string, author: string): Promise<Blueprint> => {
    return invoke("save_blueprint", { name, data, author });
  }, []);

  const getHistory = useCallback(async (limit: number): Promise<HistoryEntry[]> => {
    return invoke("get_history", { limit });
  }, []);

  const logEvent = useCallback(async (eventType: string, player: string | null, details: string): Promise<HistoryEntry> => {
    return invoke("log_event", { eventType, player, details });
  }, []);

  return useMemo(() => ({ listPlayers, savePlayer, listLocations, saveLocation, listBlueprints, saveBlueprint, getHistory, logEvent }), [listPlayers, savePlayer, listLocations, saveLocation, listBlueprints, saveBlueprint, getHistory, logEvent]);
};

// Event listener hook
export const useBotEvents = (onEvent: (event: BotEvent) => void) => {
  const unlistenRef = useRef<UnlistenFn | null>(null);

  useEffect(() => {
    const setup = async () => {
      unlistenRef.current = await listen<BotEvent>("bot://event", (event) => {
        onEvent(event.payload);
      });
    };

    setup();

    return () => {
      unlistenRef.current?.();
    };
  }, [onEvent]);
};

// Bot status polling hook
export const useBotStatusPolling = (intervalMs: number = 5000) => {
  const [status, setStatus] = useState<BotStatus>({
    connected: false,
    server: "",
    username: "",
    health: 20,
    food: 20,
    x: 0,
    y: 0,
    z: 0,
    uptime_seconds: 0,
  });

  const { getBotStatus } = useBot();

  useEffect(() => {
    const poll = async () => {
      try {
        const newStatus = await getBotStatus();
        setStatus(newStatus);
      } catch (e) {
        console.error("Failed to get bot status:", e);
      }
    };

    poll();
    const interval = setInterval(poll, intervalMs);
    return () => clearInterval(interval);
  }, [intervalMs, getBotStatus]);

  return status;
};
