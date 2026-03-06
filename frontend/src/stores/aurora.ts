import { defineStore } from "pinia";
import { ref } from "vue";
import {
  WS_BASE,
  type FullStateData,
  type KpForecast,
  type KpIndex,
  type OvationData,
  type StatusResponse,
  type SwpcAlert,
  type TonightViewlineResponse,
  type ViewlinePoint,
  type WsMessage,
  type SolarWind,
} from "@/api/client";

const RECONNECT_DELAY_MIN_MS = 1000;
const RECONNECT_DELAY_MAX_MS = 30000;

export const useAuroraStore = defineStore("aurora", () => {
  const viewline = ref<ViewlinePoint[]>([]);
  const tonightViewline = ref<TonightViewlineResponse | null>(null);
  const ovation = ref<OvationData | null>(null);
  const kpCurrent = ref<KpIndex[]>([]);
  const kpForecast = ref<KpForecast[]>([]);
  const solarWind = ref<SolarWind[]>([]);
  const swpcAlerts = ref<SwpcAlert[]>([]);
  const noaaScales = ref<unknown | null>(null);
  const status = ref<StatusResponse | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  let ws: WebSocket | null = null;
  let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
  let reconnectDelay = RECONNECT_DELAY_MIN_MS;

  function handleWsMessage(msg: WsMessage) {
    switch (msg.type) {
      case "FullState": {
        const d: FullStateData = msg.data;
        viewline.value = d.viewline;
        tonightViewline.value = d.tonight_viewline;
        ovation.value = d.ovation;
        kpCurrent.value = d.kp_current;
        kpForecast.value = d.kp_forecast;
        solarWind.value = d.solar_wind;
        swpcAlerts.value = d.swpc_alerts;
        noaaScales.value = d.noaa_scales;
        status.value = {
          healthy: true,
          last_ovation_poll: d.last_ovation_poll,
          last_kp_poll: d.last_kp_poll,
          last_solar_wind_poll: d.last_solar_wind_poll,
          alert_active: d.alert_active,
          location: {
            name: d.location_name,
            latitude: d.location_lat,
            longitude: d.location_lon,
          },
        };
        loading.value = false;
        break;
      }
      case "KpUpdate":
        kpCurrent.value = msg.data;
        break;
      case "KpForecastUpdate":
        kpForecast.value = msg.data;
        break;
      case "SolarWindUpdate":
        solarWind.value = msg.data;
        break;
      case "ViewlineUpdate":
        viewline.value = msg.data;
        break;
      case "OvationUpdate":
        ovation.value = msg.data;
        break;
      case "SwpcAlertsUpdate":
        swpcAlerts.value = msg.data;
        break;
      case "NoaaScalesUpdate":
        noaaScales.value = msg.data;
        break;
      case "StatusUpdate":
        if (status.value) {
          status.value.alert_active = msg.data.alert_active;
          status.value.last_ovation_poll = msg.data.last_ovation_poll;
        }
        break;
    }
  }

  function scheduleReconnect() {
    reconnectTimeout = setTimeout(() => {
      reconnectDelay = Math.min(reconnectDelay * 2, RECONNECT_DELAY_MAX_MS);
      connectWebSocket();
    }, reconnectDelay);
  }

  function connectWebSocket() {
    if (ws && ws.readyState === WebSocket.OPEN) return;

    loading.value = true;
    error.value = null;

    const wsUrl = `${WS_BASE}/api/ws`;
    ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      reconnectDelay = RECONNECT_DELAY_MIN_MS;
    };

    ws.onmessage = (event: MessageEvent<string>) => {
      try {
        const msg = JSON.parse(event.data) as WsMessage;
        handleWsMessage(msg);
      } catch (e) {
        console.error("Failed to parse WebSocket message:", e);
      }
    };

    ws.onclose = () => {
      scheduleReconnect();
    };

    ws.onerror = () => {
      error.value = "WebSocket connection error";
      ws?.close();
    };
  }

  function disconnectWebSocket() {
    if (reconnectTimeout) {
      clearTimeout(reconnectTimeout);
      reconnectTimeout = null;
    }
    ws?.close();
    ws = null;
  }

  return {
    viewline,
    tonightViewline,
    ovation,
    kpCurrent,
    kpForecast,
    solarWind,
    swpcAlerts,
    noaaScales,
    status,
    loading,
    error,
    connectWebSocket,
    disconnectWebSocket,
  };
});
