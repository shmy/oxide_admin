use http_client_kit::HTTP_CLIENT;
use serde::Deserialize;

use crate::TencentLocationResponse;

#[derive(Debug, Deserialize)]
pub struct GeocoderResponse {
    pub ad_info: GeocoderResponseAdInfo,
}

#[derive(Debug, Deserialize)]
pub struct GeocoderResponseAdInfo {
    pub adcode: String,
}

pub type Codes = (u32, u32, u32);

#[tracing::instrument]
pub async fn geocoder(key: &str, location: &str) -> Option<Codes> {
    let builder = HTTP_CLIENT
        .get("https://apis.map.qq.com/ws/geocoder/v1")
        .query(&[("key", key), ("location", location)]);
    let Ok(response) = builder.send().await else {
        return None;
    };
    let Ok(suggestion_response) = response
        .json::<TencentLocationResponse<GeocoderResponse>>()
        .await
    else {
        return None;
    };
    if suggestion_response.status != 0 {
        return None;
    }
    parse_adcode(&suggestion_response.result.ad_info.adcode)
}

fn parse_adcode(adcode: &str) -> Option<Codes> {
    let province_code = adcode[0..2].parse::<u32>().ok()?;
    let mut city_code = adcode[0..4].parse::<u32>().ok()?;
    let district_code = adcode[0..6].parse::<u32>().ok()?;
    // Hong Kong and Macau
    if province_code == 81 {
        city_code = 8100;
    } else if province_code == 82 {
        city_code = 8200;
    }
    Some((province_code, city_code, district_code))
}

#[cfg(test)]
mod tests {
    use super::geocoder;

    #[tokio::test]
    async fn test_geocoder1() {
        let result = geocoder(
            "YLMBZ-5TX3Z-SXKXF-Z4K3H-IMBDS-SAB2E",
            "31.490017,104.769728",
        )
        .await;
        assert_eq!(result, Some((51, 5107, 510704)));
    }
}
