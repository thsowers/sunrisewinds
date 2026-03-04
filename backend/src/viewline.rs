use std::collections::BTreeMap;

use crate::models::{OvationResponse, ViewlinePoint};

/// Earth's mean radius in km.
const EARTH_RADIUS_KM: f64 = 6371.0;

/// Aurora base altitude in km — the lower/brighter edge of aurora, which
/// determines how far over the horizon it can be seen.
const AURORA_ALTITUDE_KM: f64 = 100.0;

/// Geomagnetic north pole (IGRF-13 epoch 2025 approximation).
const GEOMAG_POLE_LAT_DEG: f64 = 80.7;
const GEOMAG_POLE_LON_DEG: f64 = -72.7;

fn to_rad(deg: f64) -> f64 {
    deg * std::f64::consts::PI / 180.0
}

fn to_deg(rad: f64) -> f64 {
    rad * 180.0 / std::f64::consts::PI
}

/// Normalize longitude to -180..180.
fn normalize_lon(lon: f64) -> f64 {
    let mut l = lon % 360.0;
    if l > 180.0 {
        l -= 360.0;
    } else if l < -180.0 {
        l += 360.0;
    }
    l
}

/// Angular distance (radians) on a sphere between two lat/lon points (in radians).
fn angular_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    (lat1.sin() * lat2.sin() + lat1.cos() * lat2.cos() * (lon2 - lon1).cos()).acos()
}

/// Horizon viewing angle (radians) — how far an observer at ground level
/// can see an object at AURORA_ALTITUDE_KM due to Earth's curvature.
fn horizon_offset_rad() -> f64 {
    (EARTH_RADIUS_KM / (EARTH_RADIUS_KM + AURORA_ALTITUDE_KM)).acos()
}

/// Computes the aurora viewline from OVATION data.
///
/// Algorithm:
/// 1. For each longitude, find the aurora edge (lowest latitude with
///    probability >= threshold) — Northern Hemisphere only.
/// 2. Compute each edge point's angular distance from the geomagnetic pole.
/// 3. Take the maximum distance (furthest equatorward extent of the oval).
/// 4. Add the horizon viewing offset to get the viewline distance.
/// 5. Generate the viewline as a circle at that distance from the geomagnetic
///    pole, converted to geographic coordinates.
///
/// This produces the smooth, slightly asymmetric oval matching the NOAA
/// visualization — the line dips further south over North America (near the
/// geomagnetic pole) and stays further north over Asia.
pub fn compute_viewline(ovation: &OvationResponse, threshold: f64) -> Vec<ViewlinePoint> {
    let pole_lat = to_rad(GEOMAG_POLE_LAT_DEG);
    let pole_lon = to_rad(GEOMAG_POLE_LON_DEG);

    // Step 1: Find aurora edge per longitude
    let mut lon_to_min_lat: BTreeMap<i32, f64> = BTreeMap::new();

    for coord in &ovation.coordinates {
        let lon = coord[0]; // 0..359
        let lat = coord[1];
        let probability = coord[2];

        if lat <= 0.0 {
            continue;
        }

        if probability >= threshold {
            let lon_key = lon as i32;
            let entry = lon_to_min_lat.entry(lon_key).or_insert(90.0);
            if lat < *entry {
                *entry = lat;
            }
        }
    }

    // Step 2-3: Find maximum angular distance from geomagnetic pole
    let mut max_dist: f64 = 0.0;
    for (&lon_key, &lat) in &lon_to_min_lat {
        let geo_lon = if lon_key > 180 {
            lon_key as f64 - 360.0
        } else {
            lon_key as f64
        };
        let dist = angular_distance(to_rad(lat), to_rad(geo_lon), pole_lat, pole_lon);
        if dist > max_dist {
            max_dist = dist;
        }
    }

    if max_dist == 0.0 {
        return Vec::new();
    }

    // Step 4: Add horizon offset
    let viewline_dist = max_dist + horizon_offset_rad();

    // Step 5: Generate viewline as a circle at viewline_dist from the
    // geomagnetic pole, sampled every 1° of azimuth
    let mut points: Vec<ViewlinePoint> = Vec::with_capacity(360);

    for az_deg in 0..360 {
        let az = to_rad(az_deg as f64);

        // Point at angular distance viewline_dist from pole, at azimuth az
        let lat = (pole_lat.sin() * viewline_dist.cos()
            + pole_lat.cos() * viewline_dist.sin() * az.cos())
        .asin();

        let lon = pole_lon
            + (az.sin() * viewline_dist.sin() * pole_lat.cos())
                .atan2(viewline_dist.cos() - pole_lat.sin() * lat.sin());

        let lat_deg = to_deg(lat);
        let lon_deg = normalize_lon(to_deg(lon));

        // Only include Northern Hemisphere points
        if lat_deg > 0.0 {
            points.push(ViewlinePoint {
                lon: lon_deg,
                lat: lat_deg,
            });
        }
    }

    points.sort_by(|a, b| a.lon.partial_cmp(&b.lon).unwrap());
    points
}

/// Checks whether aurora is potentially visible at the given location.
pub fn is_aurora_visible(
    viewline: &[ViewlinePoint],
    user_lat: f64,
    user_lon: f64,
) -> Option<f64> {
    if viewline.is_empty() {
        return None;
    }

    let closest = viewline
        .iter()
        .min_by(|a, b| {
            let diff_a = (a.lon - user_lon).abs();
            let diff_b = (b.lon - user_lon).abs();
            diff_a.partial_cmp(&diff_b).unwrap_or(std::cmp::Ordering::Equal)
        })?;

    if closest.lat <= user_lat {
        Some(closest.lat)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_lon() {
        assert_eq!(normalize_lon(0.0), 0.0);
        assert_eq!(normalize_lon(270.0), -90.0);
        assert_eq!(normalize_lon(359.0), -1.0);
        assert_eq!(normalize_lon(-200.0), 160.0);
    }

    #[test]
    fn test_horizon_offset() {
        let offset_deg = to_deg(horizon_offset_rad());
        // For 100km altitude, offset should be ~10.1°
        assert!(offset_deg > 9.0 && offset_deg < 11.0, "offset was {}", offset_deg);
    }

    #[test]
    fn test_viewline_is_smooth_circle() {
        // Create a synthetic OVATION grid with aurora at 65°N everywhere
        let mut coordinates = Vec::new();
        for lon in 0..360 {
            for lat in 0..=90 {
                let prob = if lat >= 63 && lat <= 70 { 10.0 } else { 0.0 };
                coordinates.push([lon as f64, lat as f64, prob]);
            }
        }

        let ovation = OvationResponse {
            observation_time: "2024-01-01".to_string(),
            forecast_time: "2024-01-01".to_string(),
            coordinates,
        };

        let viewline = compute_viewline(&ovation, 5.0);
        assert!(!viewline.is_empty());

        // The viewline should be smooth — check that adjacent points
        // don't jump more than a few degrees in latitude
        for i in 1..viewline.len() {
            let lat_diff = (viewline[i].lat - viewline[i - 1].lat).abs();
            assert!(
                lat_diff < 3.0,
                "Jagged viewline: lat diff {} between points {} and {}",
                lat_diff,
                i - 1,
                i
            );
        }
    }

    #[test]
    fn test_viewline_asymmetry() {
        // With a uniform aurora oval, the viewline should dip further south
        // near the geomagnetic pole's longitude (-73°W) than on the opposite side
        let mut coordinates = Vec::new();
        for lon in 0..360 {
            for lat in 0..=90 {
                let prob = if lat >= 63 && lat <= 70 { 10.0 } else { 0.0 };
                coordinates.push([lon as f64, lat as f64, prob]);
            }
        }

        let ovation = OvationResponse {
            observation_time: "2024-01-01".to_string(),
            forecast_time: "2024-01-01".to_string(),
            coordinates,
        };

        let viewline = compute_viewline(&ovation, 5.0);

        // Find viewline lat near Minneapolis (-93) and near Moscow (37)
        let near_us = viewline
            .iter()
            .min_by(|a, b| (a.lon - (-93.0)).abs().partial_cmp(&(b.lon - (-93.0)).abs()).unwrap())
            .unwrap();
        let near_russia = viewline
            .iter()
            .min_by(|a, b| (a.lon - 37.0).abs().partial_cmp(&(b.lon - 37.0).abs()).unwrap())
            .unwrap();

        assert!(
            near_us.lat < near_russia.lat,
            "US viewline ({:.1}°N) should be further south than Russia ({:.1}°N)",
            near_us.lat,
            near_russia.lat,
        );
    }

    #[test]
    fn test_is_aurora_visible() {
        let viewline = vec![
            ViewlinePoint { lon: -94.0, lat: 48.0 },
            ViewlinePoint { lon: -93.0, lat: 46.0 },
            ViewlinePoint { lon: -92.0, lat: 47.0 },
        ];

        assert!(is_aurora_visible(&viewline, 45.0, -93.0).is_none());
        assert!(is_aurora_visible(&viewline, 47.0, -93.0).is_some());
    }

    #[test]
    fn test_empty_viewline() {
        assert!(is_aurora_visible(&[], 45.0, -93.0).is_none());
    }
}
