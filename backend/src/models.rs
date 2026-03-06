use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// --- OVATION Aurora ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OvationResponse {
    #[serde(rename = "Observation Time")]
    pub observation_time: String,
    #[serde(rename = "Forecast Time")]
    pub forecast_time: String,
    pub coordinates: Vec<[f64; 3]>, // [lon, lat, probability]
}

// --- Kp Index (1-minute) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpIndex {
    pub time_tag: String,
    pub kp_index: f64,
    pub estimated_kp: Option<f64>,
    pub kp: Option<String>,
}

// --- Kp Forecast ---
// The NOAA forecast endpoint returns an array of arrays (first row is headers).

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpForecast {
    pub time_tag: String,
    pub kp: f64,
    pub observed: String,
    pub noaa_scale: String,
}

// --- Solar Wind ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolarWind {
    pub time_tag: String,
    pub speed: f64,
    pub density: f64,
    pub bz: f64,
    pub bt: f64,
}

// --- Viewline ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewlinePoint {
    pub lon: f64,
    pub lat: f64,
}

// --- SWPC Alerts ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwpcAlert {
    pub product_id: String,
    pub issue_datetime: String,
    pub message: String,
}

// --- Alert ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub timestamp: DateTime<Utc>,
    pub alert_type: AlertType,
    pub viewline_lat: f64,
    pub user_lat: f64,
    pub kp: f64,
    pub notified_via: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    AuroraVisible,
    KpThresholdExceeded,
}

impl std::fmt::Display for AlertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertType::AuroraVisible => write!(f, "aurora_visible"),
            AlertType::KpThresholdExceeded => write!(f, "kp_threshold_exceeded"),
        }
    }
}

// --- Tonight's Viewline ---

#[derive(Debug, Clone, Serialize)]
pub struct TonightViewlineResponse {
    pub viewline: Vec<ViewlinePoint>,
    pub max_kp: f64,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
}

// --- API response wrappers ---

#[derive(Debug, Clone, Serialize)]
pub struct StatusResponse {
    pub healthy: bool,
    pub last_ovation_poll: Option<DateTime<Utc>>,
    pub last_kp_poll: Option<DateTime<Utc>>,
    pub last_solar_wind_poll: Option<DateTime<Utc>>,
    pub alert_active: bool,
    pub location: LocationInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct LocationInfo {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
}

// --- WebSocket Messages ---

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
#[allow(clippy::large_enum_variant)]
pub enum WsMessage {
    FullState(FullStateData),
    KpUpdate(Vec<KpIndex>),
    KpForecastUpdate(Vec<KpForecast>),
    SolarWindUpdate(Vec<SolarWind>),
    ViewlineUpdate(Vec<ViewlinePoint>),
    OvationUpdate(OvationResponse),
    SwpcAlertsUpdate(Vec<SwpcAlert>),
    NoaaScalesUpdate(serde_json::Value),
    StatusUpdate(StatusUpdateData),
}

#[derive(Debug, Clone, Serialize)]
pub struct FullStateData {
    pub viewline: Vec<ViewlinePoint>,
    pub tonight_viewline: Option<TonightViewlineResponse>,
    pub ovation: Option<OvationResponse>,
    pub kp_current: Vec<KpIndex>,
    pub kp_forecast: Vec<KpForecast>,
    pub solar_wind: Vec<SolarWind>,
    pub swpc_alerts: Vec<SwpcAlert>,
    pub noaa_scales: Option<serde_json::Value>,
    pub alert_active: bool,
    pub last_ovation_poll: Option<DateTime<Utc>>,
    pub last_kp_poll: Option<DateTime<Utc>>,
    pub last_solar_wind_poll: Option<DateTime<Utc>>,
    pub location_name: String,
    pub location_lat: f64,
    pub location_lon: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusUpdateData {
    pub alert_active: bool,
    pub last_ovation_poll: Option<DateTime<Utc>>,
}
