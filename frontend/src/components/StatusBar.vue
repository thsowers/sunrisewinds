<script setup lang="ts">
import { computed } from 'vue'
import { useAuroraStore } from '@/stores/aurora'

const store = useAuroraStore()

const connectionStatus = computed(() => {
  if (store.loading) return 'loading'
  if (store.error) return 'error'
  if (store.status) return 'connected'
  return 'disconnected'
})

const alertStatus = computed(() => store.status?.alert_active ?? false)

function formatTime(iso: string | null): string {
  if (!iso) return 'never'
  return new Date(iso).toLocaleTimeString()
}
</script>

<template>
  <div class="status-bar">
    <div class="status-item">
      <span
        class="status-dot"
        :class="{
          connected: connectionStatus === 'connected',
          loading: connectionStatus === 'loading',
          error: connectionStatus === 'error',
        }"
      ></span>
      <span class="status-text">{{ connectionStatus }}</span>
    </div>

    <div v-if="store.status" class="status-item">
      <span class="status-label">Location:</span>
      {{ store.status.location.name }}
      ({{ store.status.location.latitude }}°N)
    </div>

    <div v-if="store.status" class="status-item">
      <span class="status-label">Last update:</span>
      {{ formatTime(store.status.last_ovation_poll) }}
    </div>

    <div v-if="alertStatus" class="status-item alert-active">AURORA ALERT ACTIVE</div>
  </div>
</template>

<style scoped>
.status-bar {
  display: flex;
  align-items: center;
  gap: 24px;
  padding: 8px 16px;
  background: #1a1a2e;
  border-top: 1px solid #2a2a4a;
  font-size: 12px;
  color: #999;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #666;
}

.status-dot.connected {
  background: #69f0ae;
}

.status-dot.loading {
  background: #ffea00;
  animation: pulse 1s infinite;
}

.status-dot.error {
  background: #ff1744;
}

.status-label {
  color: #666;
}

.alert-active {
  color: #ff3333;
  font-weight: bold;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}
</style>
