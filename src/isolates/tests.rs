use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

use crate::test_utils::{build_app_with_db, setup_clean_db};

#[tokio::test]
#[ignore]
async fn create_isolate_valid() {
    let db = setup_clean_db().await;
    let app = build_app_with_db(db.clone());

    let create_site_payload = json!({
        "name": "2",
        "latitude_4326": 46.27095,
        "longitude_4326": 7.3349,
        "elevation_metres": 1629.0
    });
    let site_request = Request::builder()
        .method("POST")
        .uri("/api/sites")
        .header("Content-Type", "application/json")
        .body(Body::from(create_site_payload.to_string()))
        .unwrap();
    let site_response = app.clone().oneshot(site_request).await.unwrap();
    assert_eq!(site_response.status(), StatusCode::CREATED);
    let site_body = to_bytes(site_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let site: serde_json::Value = serde_json::from_slice(&site_body).unwrap();
    let site_id = site.get("id").unwrap().as_str().unwrap();

    let create_replicate_payload = json!({
        "site_id": site_id,
        "name": "P2S1-T",
        "sample_type": "Snow",
        "sampling_date": "2023-02-18"
    });
    let replicate_request = Request::builder()
        .method("POST")
        .uri("/api/site_replicates")
        .header("Content-Type", "application/json")
        .body(Body::from(create_replicate_payload.to_string()))
        .unwrap();
    let replicate_response = app.clone().oneshot(replicate_request).await.unwrap();
    assert_eq!(replicate_response.status(), StatusCode::CREATED);
    let replicate_body = to_bytes(replicate_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let replicate: serde_json::Value = serde_json::from_slice(&replicate_body).unwrap();
    let replicate_id = replicate.get("id").unwrap().as_str().unwrap();

    let create_payload = json!({
        "name": "Isolate A",
        "site_replicate_id": replicate_id,
        "taxonomy": "Pseudomonas",
        "photo": "",
        "temperature_of_isolation": 20.5,
        "media_used_for_isolation": "M9",
        "storage_location": "Isolates: A1",
        "genome_url": null
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/isolates")
        .header("Content-Type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
#[ignore]
async fn test_isolates_invalid_data() {
    let db = setup_clean_db().await;
    let app = build_app_with_db(db.clone());

    let create_site_payload = json!({
        "name": "Isolate_Parent_Site",
        "latitude_4326": 46.27095,
        "longitude_4326": 7.3349,
        "elevation_metres": 1629
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/sites")
        .header("Content-Type", "application/json")
        .body(Body::from(create_site_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let site_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let site: serde_json::Value = serde_json::from_slice(&site_bytes).unwrap();
    let site_id = site.get("id").unwrap().as_str().unwrap();

    let create_replicate_payload = json!({
        "site_id": site_id,
        "name": "Isolate_Replicate",
        "sample_type": "Snow",
        "sampling_date": "2023-02-18"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/site_replicates")
        .header("Content-Type", "application/json")
        .body(Body::from(create_replicate_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let rep_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let replicate: serde_json::Value = serde_json::from_slice(&rep_bytes).unwrap();
    let replicate_id = replicate.get("id").unwrap().as_str().unwrap();

    let invalid_temp_payload = json!({
        "name": "Isolate_Invalid_Temp",
        "site_replicate_id": replicate_id,
        "taxonomy": "Pseudomonas",
        "photo": "",
        "temperature_of_isolation": "hot",
        "media_used_for_isolation": "M9",
        "storage_location": "Isolates: Test",
        "genome_url": null
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/isolates")
        .header("Content-Type", "application/json")
        .body(Body::from(invalid_temp_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let invalid_uuid_payload = json!({
        "name": "Isolate_Invalid_UUID",
        "site_replicate_id": "not-a-uuid",
        "taxonomy": "Pseudomonas",
        "photo": "",
        "temperature_of_isolation": 20.5,
        "media_used_for_isolation": "M9",
        "storage_location": "Isolates: Test",
        "genome_url": null
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/isolates")
        .header("Content-Type", "application/json")
        .body(Body::from(invalid_uuid_payload.to_string()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
#[ignore]
async fn test_isolate_images() {
    let db = setup_clean_db().await;
    let app = build_app_with_db(db.clone());

    let create_site_payload = json!({
        "name": "Prabe_S2",
        "latitude_4326": 46.27095,
        "longitude_4326": 7.3349,
        "elevation_metres": 1629.0
    });
    let site_request = Request::builder()
        .method("POST")
        .uri("/api/sites")
        .header("Content-Type", "application/json")
        .body(Body::from(create_site_payload.to_string()))
        .unwrap();
    let site_response = app.clone().oneshot(site_request).await.unwrap();
    assert_eq!(site_response.status(), StatusCode::CREATED);
    let site_body = to_bytes(site_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let site: serde_json::Value = serde_json::from_slice(&site_body).unwrap();
    let site_id = site.get("id").unwrap().as_str().unwrap();

    let create_replicate_payload = json!({
        "site_id": site_id,
        "name": "P2S1-T2",
        "sample_type": "Snow",
        "sampling_date": "2023-02-18"
    });
    let replicate_request = Request::builder()
        .method("POST")
        .uri("/api/site_replicates")
        .header("Content-Type", "application/json")
        .body(Body::from(create_replicate_payload.to_string()))
        .unwrap();
    let replicate_response = app.clone().oneshot(replicate_request).await.unwrap();
    assert_eq!(replicate_response.status(), StatusCode::CREATED);
    let replicate_body = to_bytes(replicate_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let replicate: serde_json::Value = serde_json::from_slice(&replicate_body).unwrap();
    let replicate_id = replicate.get("id").unwrap().as_str().unwrap();

    let create_payload = json!({
        "name": "Isolate B",
        "site_replicate_id": replicate_id,
        "taxonomy": "Pseudomonas",
        "photo": "",
        "temperature_of_isolation": 20.5,
        "media_used_for_isolation": "M9",
        "storage_location": "Isolates: A1",
        "genome_url": null
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/isolates")
        .header("Content-Type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let isolate_body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let isolate: serde_json::Value = serde_json::from_slice(&isolate_body).unwrap();
    let isolate_id = isolate.get("id").unwrap().as_str().unwrap();

    let image_payload = json!({
        "photo": "data:image/png;base64,abc123"
    });
    let request = Request::builder()
        .method("PUT")
        .uri(format!("/api/isolates/{isolate_id}"))
        .header("Content-Type", "application/json")
        .body(Body::from(image_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/isolates/{}", isolate_id))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK, "Failed to get one isolate");
    let isolate_body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let isolate: serde_json::Value = serde_json::from_slice(&isolate_body).unwrap();
    let photo = isolate.get("photo").unwrap();
    assert_eq!(
        photo.as_str().unwrap(),
        "data:image/png;base64,abc123",
        "Photo should be returned in get one"
    );
    assert!(
        isolate.get("created_at").and_then(|v| v.as_str()).is_some(),
        "created_at should be serialized in get one response"
    );

    let create_second_payload = json!({
        "name": "Isolate NoPhoto",
        "site_replicate_id": replicate_id,
        "taxonomy": "Bacillus",
        "temperature_of_isolation": 4.0,
        "media_used_for_isolation": "R2A"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/isolates")
        .header("Content-Type", "application/json")
        .body(Body::from(create_second_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let second_body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let second: serde_json::Value = serde_json::from_slice(&second_body).unwrap();
    let second_id = second.get("id").unwrap().as_str().unwrap();

    let request = Request::builder()
        .method("GET")
        .uri("/api/isolates")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK, "Failed to get all isolates");
    let isolates_body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let isolates: serde_json::Value = serde_json::from_slice(&isolates_body).unwrap();
    let isolates = isolates.as_array().unwrap();

    let isolate = isolates
        .iter()
        .find(|i| i.get("id").unwrap().as_str().unwrap() == isolate_id)
        .unwrap();
    let photo = isolate.get("photo").unwrap();
    assert_eq!(
        photo.as_str(),
        None,
        "Photo should not be returned in get all"
    );
    assert_eq!(
        isolate.get("has_photo").and_then(|v| v.as_bool()),
        Some(true),
        "has_photo should be true for an isolate with a non-empty photo"
    );
    assert!(
        isolate.get("created_at").and_then(|v| v.as_str()).is_some(),
        "created_at should be serialized in list response"
    );

    let second_in_list = isolates
        .iter()
        .find(|i| i.get("id").unwrap().as_str().unwrap() == second_id)
        .unwrap();
    assert_eq!(
        second_in_list.get("has_photo").and_then(|v| v.as_bool()),
        Some(false),
        "has_photo should be false for an isolate without a photo"
    );
}
