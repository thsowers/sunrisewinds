<script setup lang="ts">
import { computed } from "vue";
import { Bar } from "vue-chartjs";
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  BarElement,
  Title,
  Tooltip,
  Legend,
} from "chart.js";
import { useAuroraStore } from "@/stores/aurora";

ChartJS.register(
  CategoryScale,
  LinearScale,
  BarElement,
  Title,
  Tooltip,
  Legend,
);

const store = useAuroraStore();

const currentKp = computed(() => {
  if (store.kpCurrent.length === 0) return null;
  return store.kpCurrent[store.kpCurrent.length - 1];
});

const kpColor = computed(() => {
  const kp = currentKp.value?.kp_index ?? 0;
  if (kp >= 7) return "#ff1744";
  if (kp >= 5) return "#ff9100";
  if (kp >= 3) return "#ffea00";
  return "#69f0ae";
});

const forecastChartData = computed(() => {
  const forecast = store.kpForecast.slice(0, 24);
  return {
    labels: forecast.map((f) => {
      const date = new Date(f.time_tag);
      return date.toLocaleString("en-US", {
        month: "short",
        day: "numeric",
        hour: "numeric",
      });
    }),
    datasets: [
      {
        label: "Kp Forecast",
        data: forecast.map((f) => f.kp),
        backgroundColor: forecast.map((f) => {
          if (f.kp >= 7) return "#ff1744";
          if (f.kp >= 5) return "#ff9100";
          if (f.kp >= 3) return "#ffea00";
          return "#69f0ae";
        }),
        borderRadius: 3,
      },
    ],
  };
});

const chartOptions = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: { display: false },
    title: { display: false },
  },
  scales: {
    y: {
      min: 0,
      max: 9,
      ticks: { color: "#aaa" },
      grid: { color: "rgba(255,255,255,0.1)" },
    },
    x: {
      ticks: {
        color: "#aaa",
        maxTicksLimit: 8,
        maxRotation: 45,
      },
      grid: { display: false },
    },
  },
};
</script>

<template>
  <div class="kp-panel">
    <h3>Kp Index</h3>

    <div class="kp-gauge">
      <div class="kp-value" :style="{ color: kpColor }">
        {{ currentKp ? currentKp.kp_index.toFixed(1) : "--" }}
      </div>
      <div class="kp-label">Current Kp</div>
      <div v-if="currentKp" class="kp-time">
        {{ new Date(currentKp.time_tag).toLocaleTimeString() }}
      </div>
    </div>

    <div class="kp-forecast">
      <h4>3-Day Forecast</h4>
      <div class="chart-container">
        <Bar
          v-if="store.kpForecast.length > 0"
          :data="forecastChartData"
          :options="chartOptions"
        />
        <div v-else class="no-data">No forecast data</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.kp-panel {
  padding: 16px;
}

.kp-panel h3 {
  margin: 0 0 12px 0;
  color: #e0e0e0;
  font-size: 14px;
  text-transform: uppercase;
  letter-spacing: 1px;
}

.kp-gauge {
  text-align: center;
  margin-bottom: 16px;
}

.kp-value {
  font-size: 48px;
  font-weight: bold;
  line-height: 1;
}

.kp-label {
  color: #999;
  font-size: 12px;
  margin-top: 4px;
}

.kp-time {
  color: #666;
  font-size: 11px;
  margin-top: 2px;
}

.kp-forecast h4 {
  margin: 0 0 8px 0;
  color: #bbb;
  font-size: 12px;
}

.chart-container {
  height: 160px;
}

.no-data {
  color: #666;
  text-align: center;
  padding: 40px 0;
  font-size: 13px;
}
</style>
