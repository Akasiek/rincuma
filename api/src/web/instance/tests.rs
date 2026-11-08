#![allow(clippy::panic_in_result_fn)]

use std::path::PathBuf;

use axum::{Json, extract::State};

use super::get_instance;
use crate::{config::AppConfig, web::test_support::empty_state};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn reports_disabled_registration_by_default() -> TestResult {
    let state = empty_state(AppConfig::for_test(PathBuf::new())).await?;

    let Json(instance) = get_instance(State(state)).await;

    assert!(!instance.registration_enabled);
    Ok(())
}

#[tokio::test]
async fn reports_enabled_registration() -> TestResult {
    let config = AppConfig::for_test(PathBuf::new()).with_registration_enabled();
    let state = empty_state(config).await?;

    let Json(instance) = get_instance(State(state)).await;

    assert!(instance.registration_enabled);
    Ok(())
}
