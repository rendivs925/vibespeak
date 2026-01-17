// API presentation layer - REST endpoints

use crate::application::services::VoiceCommandProcessor;
use crate::application::use_cases::{ProcessVoiceCommandUseCase, GetSystemStatusUseCase};
use crate::application::dtos::{VoiceCommandRequest, SystemStatusResponse};
use crate::shared::Result;
use std::sync::Arc;
use warp::Filter;

pub struct ApiInterface {
    voice_processor: Arc<VoiceCommandProcessor>,
    process_use_case: ProcessVoiceCommandUseCase,
    status_use_case: GetSystemStatusUseCase,
}

impl ApiInterface {
    pub fn new(voice_processor: Arc<VoiceCommandProcessor>) -> Self {
        let process_use_case = ProcessVoiceCommandUseCase::new(voice_processor.clone());
        let status_use_case = GetSystemStatusUseCase::new(voice_processor.clone());

        Self {
            voice_processor,
            process_use_case,
            status_use_case,
        }
    }

    pub fn routes(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        self.voice_routes()
            .or(self.system_routes())
            .or(self.workflow_routes())
            .or(self.plugin_routes())
    }

    fn voice_routes(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let process_voice = warp::path("voice")
            .and(warp::path("process"))
            .and(warp::post())
            .and(warp::body::json())
            .and_then({
                let use_case = self.process_use_case.clone();
                move |request: VoiceCommandRequest| {
                    let use_case = use_case.clone();
                    async move {
                        match use_case.execute(request).await {
                            Ok(response) => Ok(warp::reply::json(&response)),
                            Err(e) => Ok(warp::reply::json(&serde_json::json!({
                                "success": false,
                                "error": e.to_string()
                            }))),
                        }
                    }
                }
            });

        let voice_test = warp::path("voice")
            .and(warp::path("test"))
            .and(warp::post())
            .and(warp::body::json())
            .and_then({
                let processor = self.voice_processor.clone();
                move |request: serde_json::Value| {
                    let processor = processor.clone();
                    async move {
                        // Simple test - just acknowledge
                        Ok(warp::reply::json(&serde_json::json!({
                            "success": true,
                            "message": "Voice test acknowledged",
                            "request": request
                        })))
                    }
                }
            });

        process_voice.or(voice_test)
    }

    fn system_routes(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let status = warp::path("system")
            .and(warp::path("status"))
            .and(warp::get())
            .and_then({
                let use_case = self.status_use_case.clone();
                move || {
                    let use_case = use_case.clone();
                    async move {
                        match use_case.execute().await {
                            Ok(status) => Ok(warp::reply::json(&status)),
                            Err(e) => Ok(warp::reply::json(&serde_json::json!({
                                "error": e.to_string()
                            }))),
                        }
                    }
                }
            });

        let info = warp::path("system")
            .and(warp::path("info"))
            .and(warp::get())
            .map(|| {
                warp::reply::json(&serde_json::json!({
                    "name": env!("CARGO_PKG_NAME"),
                    "version": env!("CARGO_PKG_VERSION"),
                    "description": env!("CARGO_PKG_DESCRIPTION"),
                    "authors": env!("CARGO_PKG_AUTHORS")
                }))
            });

        status.or(info)
    }

    fn workflow_routes(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let list_workflows = warp::path("workflows")
            .and(warp::get())
            .map(|| {
                // TODO: Get from repository
                warp::reply::json(&serde_json::json!({
                    "workflows": [],
                    "message": "Workflow listing not yet implemented"
                }))
            });

        let create_workflow = warp::path("workflows")
            .and(warp::post())
            .and(warp::body::json())
            .and_then({
                let processor = self.voice_processor.clone();
                move |request: serde_json::Value| {
                    let processor = processor.clone();
                    async move {
                        // TODO: Implement workflow creation
                        Ok(warp::reply::json(&serde_json::json!({
                            "success": false,
                            "message": "Workflow creation not yet implemented",
                            "request": request
                        })))
                    }
                }
            });

        list_workflows.or(create_workflow)
    }

    fn plugin_routes(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let list_plugins = warp::path("plugins")
            .and(warp::get())
            .and_then({
                let processor = self.voice_processor.clone();
                move || {
                    let processor = processor.clone();
                    async move {
                        let plugins = processor.get_available_plugins();
                        Ok(warp::reply::json(&serde_json::json!({
                            "plugins": plugins
                        })))
                    }
                }
            });

        list_plugins
    }
}
