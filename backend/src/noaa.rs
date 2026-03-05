use anyhow::{Context, Result};
use reqwest::Client;
use tracing::{debug, warn};

use crate::models::{KpForecast, KpIndex, OvationResponse, SolarWind, SwpcAlert};

const OVATION_URL: &str = "https://services.swpc.noaa.gov/json/ovation_aurora_latest.json";
const KP_INDEX_URL: &str = "https://services.swpc.noaa.gov/json/planetary_k_index_1m.json";
const KP_FORECAST_URL: &str =
    "https://services.swpc.noaa.gov/products/noaa-planetary-k-index-forecast.json";
const SOLAR_WIND_URL: &str =
    "https://services.swpc.noaa.gov/products/geospace/propagated-solar-wind-1-hour.json";
const SWPC_ALERTS_URL: &str = "https://services.swpc.noaa.gov/products/alerts.json";
const NOAA_SCALES_URL: &str = "https://services.swpc.noaa.gov/products/noaa-scales.json";

pub struct NoaaClient {
    client: Client,
}

impl NoaaClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn fetch_ovation(&self) -> Result<OvationResponse> {
        debug!("Fetching OVATION aurora data");
        let resp = self
            .client
            .get(OVATION_URL)
            .send()
            .await
            .context("Failed to fetch OVATION data")?;

        let data: OvationResponse = resp.json().await.context("Failed to parse OVATION data")?;
        debug!(
            observation_time = %data.observation_time,
            coordinates_count = data.coordinates.len(),
            "OVATION data fetched"
        );
        Ok(data)
    }

    pub async fn fetch_kp_index(&self) -> Result<Vec<KpIndex>> {
        debug!("Fetching Kp index data");
        let resp = self
            .client
            .get(KP_INDEX_URL)
            .send()
            .await
            .context("Failed to fetch Kp index")?;

        let data: Vec<KpIndex> = resp.json().await.context("Failed to parse Kp index")?;
        debug!(count = data.len(), "Kp index data fetched");
        Ok(data)
    }

    pub async fn fetch_kp_forecast(&self) -> Result<Vec<KpForecast>> {
        debug!("Fetching Kp forecast data");
        let resp = self
            .client
            .get(KP_FORECAST_URL)
            .send()
            .await
            .context("Failed to fetch Kp forecast")?;

        // The forecast endpoint returns an array of arrays where the first row is headers.
        // Fields can be null (e.g. noaa_scale), so we parse as serde_json::Value.
        let raw: Vec<Vec<serde_json::Value>> =
            resp.json().await.context("Failed to parse Kp forecast")?;

        let forecasts: Vec<KpForecast> = raw
            .into_iter()
            .skip(1) // skip header row
            .filter_map(|row| {
                if row.len() < 4 {
                    warn!("Skipping malformed Kp forecast row: {:?}", row);
                    return None;
                }
                Some(KpForecast {
                    time_tag: row[0].as_str().unwrap_or_default().to_string(),
                    kp: row[1]
                        .as_str()
                        .and_then(|s| s.parse().ok())
                        .or_else(|| row[1].as_f64())
                        .unwrap_or(0.0),
                    observed: row[2].as_str().unwrap_or_default().to_string(),
                    noaa_scale: row[3].as_str().unwrap_or("").to_string(),
                })
            })
            .collect();

        debug!(count = forecasts.len(), "Kp forecast data fetched");
        Ok(forecasts)
    }

    pub async fn fetch_solar_wind(&self) -> Result<Vec<SolarWind>> {
        debug!("Fetching solar wind data");
        let resp = self
            .client
            .get(SOLAR_WIND_URL)
            .send()
            .await
            .context("Failed to fetch solar wind")?;

        // Solar wind endpoint returns array of arrays with header row.
        // Columns: time_tag[0], speed[1], density[2], temperature[3], bx[4], by[5], bz[6], bt[7], ...
        // Fields can be null, so we parse as serde_json::Value.
        let raw: Vec<Vec<serde_json::Value>> =
            resp.json().await.context("Failed to parse solar wind")?;

        let data: Vec<SolarWind> = raw
            .into_iter()
            .skip(1) // skip header row
            .filter_map(|row| {
                if row.len() < 8 {
                    warn!("Skipping malformed solar wind row (len={})", row.len());
                    return None;
                }
                let parse_f64 = |v: &serde_json::Value| -> f64 {
                    v.as_str()
                        .and_then(|s| s.parse().ok())
                        .or_else(|| v.as_f64())
                        .unwrap_or(0.0)
                };
                Some(SolarWind {
                    time_tag: row[0].as_str().unwrap_or_default().to_string(),
                    speed: parse_f64(&row[1]),
                    density: parse_f64(&row[2]),
                    bz: parse_f64(&row[6]),
                    bt: parse_f64(&row[7]),
                })
            })
            .collect();

        debug!(count = data.len(), "Solar wind data fetched");
        Ok(data)
    }

    pub async fn fetch_swpc_alerts(&self) -> Result<Vec<SwpcAlert>> {
        debug!("Fetching SWPC alerts");
        let resp = self
            .client
            .get(SWPC_ALERTS_URL)
            .send()
            .await
            .context("Failed to fetch SWPC alerts")?;

        let data: Vec<SwpcAlert> = resp.json().await.context("Failed to parse SWPC alerts")?;
        debug!(count = data.len(), "SWPC alerts fetched");
        Ok(data)
    }

    pub async fn fetch_noaa_scales(&self) -> Result<serde_json::Value> {
        debug!("Fetching NOAA scales");
        let resp = self
            .client
            .get(NOAA_SCALES_URL)
            .send()
            .await
            .context("Failed to fetch NOAA scales")?;

        let data: serde_json::Value =
            resp.json().await.context("Failed to parse NOAA scales")?;
        debug!("NOAA scales fetched");
        Ok(data)
    }
}
