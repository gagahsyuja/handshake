use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Deserialize)]
struct NominatimResponse {
    lat: String,
    lon: String,
    display_name: String,
}

#[derive(Debug, Serialize)]
pub struct GeocodeResult {
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
}

pub async fn geocode(address: &str) -> Result<GeocodeResult, String> {
    let nominatim_url =
        env::var("NOMINATIM_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let client = reqwest::Client::new();
    let url = format!(
        "{}/search?q={}&format=json&limit=1",
        nominatim_url,
        urlencoding::encode(address)
    );

    let response = client
        .get(&url)
        .header("User-Agent", "Handshake-Marketplace/1.0")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let results: Vec<NominatimResponse> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if let Some(result) = results.first() {
        Ok(GeocodeResult {
            latitude: result.lat.parse().map_err(|_| "Invalid latitude")?,
            longitude: result.lon.parse().map_err(|_| "Invalid longitude")?,
            address: result.display_name.clone(),
        })
    } else {
        Err("Address not found".to_string())
    }
}

pub async fn reverse_geocode_from_coord(lat: f64, lon: f64) -> Result<GeocodeResult, String> {
    let nominatim_url =
        env::var("NOMINATIM_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let client = reqwest::Client::new();
    let url = format!(
        "{}/reverse?lat={}&lon={}&format=json",
        nominatim_url, lat, lon
    );

    let response = client
        .get(&url)
        .header("User-Agent", "Handshake-Marketplace/1.0")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let result: NominatimResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(GeocodeResult {
        latitude: result.lat.parse().map_err(|_| "Invalid latitude")?,
        longitude: result.lon.parse().map_err(|_| "Invalid longitude")?,
        address: result.display_name,
    })
}
