use axum::{http::StatusCode, response::Json};

use crate::{
    api::models::{ErrorResponse, SuccessResponse},
    data::{get_default_airports, get_default_cargo_types},
};

pub async fn health_check() -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    Ok(Json(SuccessResponse {
        message: "KZRK Game API is running".to_string(),
        data: None,
    }))
}

pub async fn get_available_airports()
-> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let airports = get_default_airports();

    let airport_list: Vec<serde_json::Value> = airports
        .values()
        .map(|airport| {
            serde_json::json!({
                "id": airport.id,
                "name": airport.name,
                "coordinates": airport.coordinates,
                "base_fuel_price": airport.base_fuel_price
            })
        })
        .collect();

    Ok(Json(serde_json::json!(airport_list)))
}

pub async fn get_available_cargo()
-> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let cargo_types = get_default_cargo_types();

    let cargo_list: Vec<serde_json::Value> = cargo_types
        .values()
        .map(|cargo_type| {
            serde_json::json!({
                "id": cargo_type.id,
                "name": cargo_type.name,
                "base_price": cargo_type.base_price,
                "weight_per_unit": cargo_type.weight_per_unit,
                "volatility": cargo_type.volatility
            })
        })
        .collect();

    Ok(Json(serde_json::json!(cargo_list)))
}
