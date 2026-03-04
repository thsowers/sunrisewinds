<script setup lang="ts">
import { computed } from 'vue'
import { Line } from 'vue-chartjs'
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  Filler,
} from 'chart.js'
import { useAuroraStore } from '@/stores/aurora'

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend, Filler)

const store = useAuroraStore()

const latestWind = computed(() => {
  if (store.solarWind.length === 0) return null
  return store.solarWind[store.solarWind.length - 1]
})

function makeChartData(field: 'speed' | 'density' | 'bz', color: string, label: string) {
  return computed(() => ({
    labels: store.solarWind.map((sw) => {
      const d = new Date(sw.time_tag)
      return d.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' })
    }),
    datasets: [
      {
        label,
        data: store.solarWind.map((sw) => sw[field]),
        borderColor: color,
        backgroundColor: color + '20',
        fill: true,
        tension: 0.3,
        pointRadius: 0,
        borderWidth: 2,
      },
    ],
  }))
}

const speedData = makeChartData('speed', '#4fc3f7', 'Speed (km/s)')
const densityData = makeChartData('density', '#ff9100', 'Density (p/cm³)')
const bzData = makeChartData('bz', '#e040fb', 'Bz (nT)')

const chartOptions = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: { display: false },
  },
  scales: {
    y: {
      ticks: { color: '#aaa', font: { size: 10 } },
      grid: { color: 'rgba(255,255,255,0.05)' },
    },
    x: {
      ticks: { color: '#aaa', maxTicksLimit: 6, font: { size: 10 } },
      grid: { display: false },
    },
  },
}
</script>

<template>
  <div class="solar-wind-panel">
    <h3>Solar Wind</h3>

    <div v-if="latestWind" class="current-values">
      <div class="value-item">
        <span class="value" style="color: #4fc3f7">{{ latestWind.speed.toFixed(0) }}</span>
        <span class="unit">km/s</span>
      </div>
      <div class="value-item">
        <span class="value" style="color: #ff9100">{{ latestWind.density.toFixed(1) }}</span>
        <span class="unit">p/cm³</span>
      </div>
      <div class="value-item">
        <span class="value" style="color: #e040fb">{{ latestWind.bz.toFixed(1) }}</span>
        <span class="unit">Bz nT</span>
      </div>
    </div>

    <div class="charts">
      <div class="mini-chart">
        <div class="chart-label">Speed</div>
        <div class="chart-container">
          <Line v-if="store.solarWind.length > 0" :data="speedData" :options="chartOptions" />
        </div>
      </div>
      <div class="mini-chart">
        <div class="chart-label">Density</div>
        <div class="chart-container">
          <Line v-if="store.solarWind.length > 0" :data="densityData" :options="chartOptions" />
        </div>
      </div>
      <div class="mini-chart">
        <div class="chart-label">Bz</div>
        <div class="chart-container">
          <Line v-if="store.solarWind.length > 0" :data="bzData" :options="chartOptions" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.solar-wind-panel {
  padding: 16px;
}

.solar-wind-panel h3 {
  margin: 0 0 12px 0;
  color: #e0e0e0;
  font-size: 14px;
  text-transform: uppercase;
  letter-spacing: 1px;
}

.current-values {
  display: flex;
  justify-content: space-around;
  margin-bottom: 16px;
}

.value-item {
  text-align: center;
}

.value {
  font-size: 24px;
  font-weight: bold;
  display: block;
}

.unit {
  color: #888;
  font-size: 11px;
}

.charts {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.mini-chart {
  position: relative;
}

.chart-label {
  color: #888;
  font-size: 11px;
  margin-bottom: 2px;
}

.chart-container {
  height: 80px;
}
</style>
