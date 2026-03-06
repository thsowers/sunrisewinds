<script setup lang="ts">
import { computed, ref } from "vue";
import { useAuroraStore } from "@/stores/aurora";
import type { SwpcAlert } from "@/api/client";

const store = useAuroraStore();

const expandedIds = ref<Set<string>>(new Set());

// Sort newest first
const sortedAlerts = computed(() =>
  [...store.swpcAlerts].sort(
    (a, b) =>
      new Date(b.issue_datetime).getTime() -
      new Date(a.issue_datetime).getTime(),
  ),
);

function toggleExpand(productId: string) {
  if (expandedIds.value.has(productId)) {
    expandedIds.value.delete(productId);
  } else {
    expandedIds.value.add(productId);
  }
  // Trigger reactivity
  expandedIds.value = new Set(expandedIds.value);
}

function isExpanded(productId: string): boolean {
  return expandedIds.value.has(productId);
}

function alertCategory(
  alert: SwpcAlert,
): "geomagnetic" | "solar-radiation" | "radio" | "other" {
  const id = alert.product_id.toUpperCase();
  const msg = alert.message.toUpperCase();
  if (
    id.includes("WAR") ||
    msg.includes("GEOMAGNETIC") ||
    msg.includes("G-SCALE")
  ) {
    return "geomagnetic";
  }
  if (
    msg.includes("SOLAR RADIATION") ||
    msg.includes("S-SCALE") ||
    msg.includes("PROTON")
  ) {
    return "solar-radiation";
  }
  if (
    msg.includes("RADIO") ||
    msg.includes("R-SCALE") ||
    msg.includes("BLACKOUT")
  ) {
    return "radio";
  }
  return "other";
}

function alertBadgeLabel(alert: SwpcAlert): string {
  const msg = alert.message;
  const watchMatch = msg.match(/\b(Watch|Warning|Alert|Summary|Forecast)\b/i);
  return watchMatch?.[1] ?? "Notice";
}

function messageSummary(message: string): string {
  // Extract the first meaningful content line from the SWPC message
  const lines = message
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l && !l.startsWith(":") && !l.startsWith("$"));
  return lines[0] || message.slice(0, 120);
}

function formatDateTime(iso: string): string {
  try {
    return new Date(iso).toLocaleString(undefined, {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
      timeZoneName: "short",
    });
  } catch {
    return iso;
  }
}
</script>

<template>
  <div class="swpc-alerts-panel">
    <div class="panel-header">
      <span class="panel-title">SWPC Alerts</span>
      <span class="alert-count" v-if="sortedAlerts.length > 0">{{
        sortedAlerts.length
      }}</span>
    </div>

    <div v-if="sortedAlerts.length === 0" class="empty-state">
      No active alerts
    </div>

    <div v-else class="alerts-feed">
      <div
        v-for="alert in sortedAlerts"
        :key="alert.product_id"
        class="alert-item"
        :class="alertCategory(alert)"
      >
        <div class="alert-header" @click="toggleExpand(alert.product_id)">
          <div class="alert-meta">
            <span class="alert-badge" :class="alertCategory(alert)">
              {{ alertBadgeLabel(alert) }}
            </span>
            <span class="alert-time">{{
              formatDateTime(alert.issue_datetime)
            }}</span>
          </div>
          <div class="alert-summary">{{ messageSummary(alert.message) }}</div>
          <span class="expand-toggle">{{
            isExpanded(alert.product_id) ? "▲" : "▼"
          }}</span>
        </div>

        <div v-if="isExpanded(alert.product_id)" class="alert-body">
          <pre class="alert-message">{{ alert.message }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.swpc-alerts-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #0d0d1a;
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  background: #1a1a2e;
  border-bottom: 1px solid #2a2a4a;
}

.panel-title {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: #aaa;
}

.alert-count {
  font-size: 11px;
  background: #2a2a4a;
  color: #aaa;
  border-radius: 10px;
  padding: 1px 7px;
}

.empty-state {
  padding: 20px 14px;
  color: #555;
  font-size: 13px;
}

.alerts-feed {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1px;
  background: #2a2a4a;
}

.alert-item {
  background: #0d0d1a;
  border-left: 3px solid #444;
}

.alert-item.geomagnetic {
  border-left-color: #ff6b35;
}

.alert-item.solar-radiation {
  border-left-color: #ffd700;
}

.alert-item.radio {
  border-left-color: #4fc3f7;
}

.alert-item.other {
  border-left-color: #69f0ae;
}

.alert-header {
  padding: 10px 14px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 5px;
  position: relative;
}

.alert-header:hover {
  background: #12122a;
}

.alert-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.alert-badge {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  padding: 2px 7px;
  border-radius: 3px;
  background: #2a2a4a;
  color: #aaa;
}

.alert-badge.geomagnetic {
  background: rgba(255, 107, 53, 0.2);
  color: #ff8c5a;
}

.alert-badge.solar-radiation {
  background: rgba(255, 215, 0, 0.15);
  color: #ffd700;
}

.alert-badge.radio {
  background: rgba(79, 195, 247, 0.15);
  color: #4fc3f7;
}

.alert-badge.other {
  background: rgba(105, 240, 174, 0.15);
  color: #69f0ae;
}

.alert-time {
  font-size: 11px;
  color: #666;
}

.alert-summary {
  font-size: 12px;
  color: #ccc;
  line-height: 1.4;
  padding-right: 20px;
}

.expand-toggle {
  position: absolute;
  right: 12px;
  top: 12px;
  color: #555;
  font-size: 10px;
}

.alert-body {
  border-top: 1px solid #1a1a2e;
  padding: 10px 14px;
}

.alert-message {
  font-size: 11px;
  color: #888;
  white-space: pre-wrap;
  word-break: break-word;
  font-family: "Courier New", monospace;
  line-height: 1.5;
  margin: 0;
}
</style>
