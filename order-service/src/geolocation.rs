use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidpointResult {
    pub midpoint: Coordinates,
    pub distance_to_buyer_km: f64,
    pub distance_to_seller_km: f64,
    pub total_distance_km: f64,
}

/// Calculate the midpoint between two geographic coordinates
pub fn calculate_midpoint(
    buyer_lat: f64,
    buyer_lon: f64,
    seller_lat: f64,
    seller_lon: f64,
) -> MidpointResult {
    // Simple midpoint calculation (average)
    let mid_lat = (buyer_lat + seller_lat) / 2.0;
    let mid_lon = (buyer_lon + seller_lon) / 2.0;

    // Calculate distances using Haversine formula
    let distance_buyer = haversine_distance(buyer_lat, buyer_lon, mid_lat, mid_lon);
    let distance_seller = haversine_distance(seller_lat, seller_lon, mid_lat, mid_lon);
    let total_distance = haversine_distance(buyer_lat, buyer_lon, seller_lat, seller_lon);

    MidpointResult {
        midpoint: Coordinates {
            latitude: mid_lat,
            longitude: mid_lon,
        },
        distance_to_buyer_km: distance_buyer,
        distance_to_seller_km: distance_seller,
        total_distance_km: total_distance,
    }
}

/// Calculate distance between two points using the Haversine formula
/// Returns distance in kilometers
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_KM * c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_midpoint() {
        let result = calculate_midpoint(
            -6.2088, 106.8456, // Jakarta
            -6.9175, 107.6191, // Bandung
        );

        assert!(result.midpoint.latitude < -6.2);
        assert!(result.midpoint.latitude > -6.9);
        assert!(result.distance_to_buyer_km > 0.0);
        assert!(result.distance_to_seller_km > 0.0);
    }

    #[test]
    fn test_haversine_distance() {
        let distance = haversine_distance(-6.2088, 106.8456, -6.9175, 107.6191);
        assert!(distance > 140.0 && distance < 160.0);
    }
}
