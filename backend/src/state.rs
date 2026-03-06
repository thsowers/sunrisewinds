use chrono::{DateTime, Utc};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

use crate::config::AppConfig;
use crate::db::Database;
use crate::models::{
    FullStateData, KpForecast, KpIndex, OvationResponse, SolarWind, SwpcAlert,
    TonightViewlineResponse, ViewlinePoint, WsMessage,
};
use crate::notifications::NotificationManager;

/// Shared application state accessible from API handlers and polling tasks.
pub struct AppState {
    pub config: AppConfig,
    pub db: Database,
    pub notifications: NotificationManager,
    pub cache: Cache,
    pub broadcast_tx: broadcast::Sender<WsMessage>,
}

/// In-memory cache for latest data (served via API and WebSocket).
pub struct Cache {
    pub ovation: RwLock<Option<OvationResponse>>,
    pub viewline: RwLock<Vec<ViewlinePoint>>,
    pub tonight_viewline: RwLock<Option<TonightViewlineResponse>>,
    pub kp_current: RwLock<Vec<KpIndex>>,
    pub kp_forecast: RwLock<Vec<KpForecast>>,
    pub solar_wind: RwLock<Vec<SolarWind>>,
    pub swpc_alerts: RwLock<Vec<SwpcAlert>>,
    pub noaa_scales: RwLock<Option<serde_json::Value>>,
    pub last_ovation_poll: RwLock<Option<DateTime<Utc>>>,
    pub last_kp_poll: RwLock<Option<DateTime<Utc>>>,
    pub last_solar_wind_poll: RwLock<Option<DateTime<Utc>>>,
    pub last_swpc_alerts_poll: RwLock<Option<DateTime<Utc>>>,
    pub alert_active: RwLock<bool>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            ovation: RwLock::new(None),
            viewline: RwLock::new(Vec::new()),
            tonight_viewline: RwLock::new(None),
            kp_current: RwLock::new(Vec::new()),
            kp_forecast: RwLock::new(Vec::new()),
            solar_wind: RwLock::new(Vec::new()),
            swpc_alerts: RwLock::new(Vec::new()),
            noaa_scales: RwLock::new(None),
            last_ovation_poll: RwLock::new(None),
            last_kp_poll: RwLock::new(None),
            last_solar_wind_poll: RwLock::new(None),
            last_swpc_alerts_poll: RwLock::new(None),
            alert_active: RwLock::new(false),
        }
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> anyhow::Result<Arc<Self>> {
        let db = Database::new(&config.database.path)?;
        let notifications = NotificationManager::new(&config.notifications, &config.email);
        let (broadcast_tx, _) = broadcast::channel(128);

        Ok(Arc::new(Self {
            config,
            db,
            notifications,
            cache: Cache::new(),
            broadcast_tx,
        }))
    }

    /// Build a full state snapshot for WebSocket initial send.
    pub fn build_full_state(&self) -> FullStateData {
        FullStateData {
            viewline: self.cache.viewline.read().unwrap().clone(),
            tonight_viewline: self.cache.tonight_viewline.read().unwrap().clone(),
            ovation: self.cache.ovation.read().unwrap().clone(),
            kp_current: self.cache.kp_current.read().unwrap().clone(),
            kp_forecast: self.cache.kp_forecast.read().unwrap().clone(),
            solar_wind: self.cache.solar_wind.read().unwrap().clone(),
            swpc_alerts: self.cache.swpc_alerts.read().unwrap().clone(),
            noaa_scales: self.cache.noaa_scales.read().unwrap().clone(),
            alert_active: *self.cache.alert_active.read().unwrap(),
            last_ovation_poll: *self.cache.last_ovation_poll.read().unwrap(),
            last_kp_poll: *self.cache.last_kp_poll.read().unwrap(),
            last_solar_wind_poll: *self.cache.last_solar_wind_poll.read().unwrap(),
            location_name: self.config.location.name.clone(),
            location_lat: self.config.location.latitude,
            location_lon: self.config.location.longitude,
        }
    }
}
