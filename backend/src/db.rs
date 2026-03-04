use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::sync::Mutex;
use tracing::info;

use crate::models::{Alert, KpIndex, SolarWind, ViewlinePoint};

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path).context("Failed to open database")?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS kp_readings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                kp_index REAL NOT NULL,
                estimated_kp REAL
            );

            CREATE TABLE IF NOT EXISTS viewline_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                data TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS solar_wind (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                speed REAL NOT NULL,
                density REAL NOT NULL,
                bz REAL NOT NULL,
                bt REAL NOT NULL
            );

            CREATE TABLE IF NOT EXISTS alerts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                alert_type TEXT NOT NULL,
                viewline_lat REAL NOT NULL,
                user_lat REAL NOT NULL,
                kp REAL NOT NULL,
                notified_via TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_kp_timestamp ON kp_readings(timestamp);
            CREATE INDEX IF NOT EXISTS idx_solar_wind_timestamp ON solar_wind(timestamp);
            CREATE INDEX IF NOT EXISTS idx_alerts_timestamp ON alerts(timestamp);
            ",
        )
        .context("Failed to run migrations")?;

        info!("Database migrations complete");
        Ok(())
    }

    pub fn insert_kp_reading(&self, kp: &KpIndex) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO kp_readings (timestamp, kp_index, estimated_kp) VALUES (?1, ?2, ?3)",
            params![kp.time_tag, kp.kp_index, kp.estimated_kp],
        )?;
        Ok(())
    }

    pub fn insert_viewline_snapshot(
        &self,
        timestamp: &DateTime<Utc>,
        viewline: &[ViewlinePoint],
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let data = serde_json::to_string(viewline)?;
        conn.execute(
            "INSERT INTO viewline_snapshots (timestamp, data) VALUES (?1, ?2)",
            params![timestamp.to_rfc3339(), data],
        )?;
        Ok(())
    }

    pub fn insert_solar_wind(&self, sw: &SolarWind) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO solar_wind (timestamp, speed, density, bz, bt) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![sw.time_tag, sw.speed, sw.density, sw.bz, sw.bt],
        )?;
        Ok(())
    }

    pub fn insert_alert(&self, alert: &Alert) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let notified_via = alert.notified_via.join(",");
        conn.execute(
            "INSERT INTO alerts (timestamp, alert_type, viewline_lat, user_lat, kp, notified_via) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                alert.timestamp.to_rfc3339(),
                alert.alert_type.to_string(),
                alert.viewline_lat,
                alert.user_lat,
                alert.kp,
                notified_via,
            ],
        )?;
        Ok(())
    }

    pub fn get_kp_history(&self, hours: i64) -> Result<Vec<KpIndex>> {
        let conn = self.conn.lock().unwrap();
        let cutoff = Utc::now() - chrono::Duration::hours(hours);
        let mut stmt = conn.prepare(
            "SELECT timestamp, kp_index, estimated_kp FROM kp_readings WHERE timestamp >= ?1 ORDER BY timestamp ASC",
        )?;

        let rows = stmt.query_map(params![cutoff.to_rfc3339()], |row| {
            Ok(KpIndex {
                time_tag: row.get(0)?,
                kp_index: row.get(1)?,
                estimated_kp: row.get(2)?,
                kp: None,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_solar_wind_history(&self, hours: i64) -> Result<Vec<SolarWind>> {
        let conn = self.conn.lock().unwrap();
        let cutoff = Utc::now() - chrono::Duration::hours(hours);
        let mut stmt = conn.prepare(
            "SELECT timestamp, speed, density, bz, bt FROM solar_wind WHERE timestamp >= ?1 ORDER BY timestamp ASC",
        )?;

        let rows = stmt.query_map(params![cutoff.to_rfc3339()], |row| {
            Ok(SolarWind {
                time_tag: row.get(0)?,
                speed: row.get(1)?,
                density: row.get(2)?,
                bz: row.get(3)?,
                bt: row.get(4)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_recent_alerts(&self, limit: i64) -> Result<Vec<Alert>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT timestamp, alert_type, viewline_lat, user_lat, kp, notified_via FROM alerts ORDER BY timestamp DESC LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit], |row| {
            let alert_type_str: String = row.get(1)?;
            let notified_via_str: String = row.get(5)?;
            let timestamp_str: String = row.get(0)?;

            Ok((
                timestamp_str,
                alert_type_str,
                row.get::<_, f64>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, f64>(4)?,
                notified_via_str,
            ))
        })?;

        let mut results = Vec::new();
        for row in rows {
            let (timestamp_str, alert_type_str, viewline_lat, user_lat, kp, notified_via_str) =
                row?;

            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let alert_type = match alert_type_str.as_str() {
                "kp_threshold_exceeded" => crate::models::AlertType::KpThresholdExceeded,
                _ => crate::models::AlertType::AuroraVisible,
            };

            let notified_via: Vec<String> = notified_via_str
                .split(',')
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect();

            results.push(Alert {
                timestamp,
                alert_type,
                viewline_lat,
                user_lat,
                kp,
                notified_via,
            });
        }
        Ok(results)
    }
}
