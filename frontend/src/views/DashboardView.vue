<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { useAuroraStore } from '@/stores/aurora'
import AuroraMap from '@/components/AuroraMap.vue'
import KpPanel from '@/components/KpPanel.vue'
import SolarWindPanel from '@/components/SolarWindPanel.vue'
import StatusBar from '@/components/StatusBar.vue'

const store = useAuroraStore()

onMounted(() => {
  store.startPolling()
})

onUnmounted(() => {
  store.stopPolling()
})
</script>

<template>
  <div class="dashboard">
    <header class="dashboard-header">
      <h1>Sunrise Winds</h1>
      <span class="subtitle">Aurora Monitor</span>
    </header>

    <div class="dashboard-grid">
      <div class="map-panel">
        <AuroraMap />
      </div>

      <div class="side-panels">
        <div class="panel">
          <KpPanel />
        </div>
        <div class="panel">
          <SolarWindPanel />
        </div>
      </div>
    </div>

    <StatusBar />
  </div>
</template>

<style scoped>
.dashboard {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: #0d0d1a;
  color: #e0e0e0;
}

.dashboard-header {
  display: flex;
  align-items: baseline;
  gap: 12px;
  padding: 12px 16px;
  background: #1a1a2e;
  border-bottom: 1px solid #2a2a4a;
}

.dashboard-header h1 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: #fff;
}

.subtitle {
  color: #888;
  font-size: 13px;
}

.dashboard-grid {
  flex: 1;
  display: grid;
  grid-template-columns: 1fr 360px;
  gap: 1px;
  background: #2a2a4a;
  overflow: hidden;
}

.map-panel {
  background: #0d0d1a;
}

.side-panels {
  display: flex;
  flex-direction: column;
  gap: 1px;
  background: #2a2a4a;
  overflow-y: auto;
}

.panel {
  background: #0d0d1a;
}

@media (max-width: 900px) {
  .dashboard-grid {
    grid-template-columns: 1fr;
    grid-template-rows: 1fr auto;
  }

  .map-panel {
    min-height: 300px;
  }
}
</style>
