use crate::application::services::VoiceProcessingService;
use crate::infrastructure::config::SystemConfig;
use crate::shared::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;

pub struct WebServer {
    voice_service: Arc<VoiceProcessingService>,
    config: Arc<RwLock<SystemConfig>>,
}

impl WebServer {
    pub fn new(voice_service: Arc<VoiceProcessingService>, config: SystemConfig) -> Self {
        Self {
            voice_service,
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub async fn run(self, port: u16) -> Result<()> {
        // Static files route
        let static_files = warp::path("static")
            .and(warp::fs::dir("web/static"));

        // API routes
        let api = self.api_routes();

        // Main interface route
        let index = warp::path::end()
            .map(|| {
                warp::reply::html(include_str!("../../../web/index.html"))
            });

        let routes = index
            .or(static_files)
            .or(api)
            .with(warp::cors().allow_any_origin());

        tracing::info!("Starting web server on port {}", port);
        warp::serve(routes)
            .run(([127, 0, 0, 1], port))
            .await;

        Ok(())
    }

    fn api_routes(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let config_get = warp::path("api")
            .and(warp::path("config"))
            .and(warp::get())
            .and(with_config(self.config.clone()))
            .and_then(get_config);

        let config_post = warp::path("api")
            .and(warp::path("config"))
            .and(warp::post())
            .and(warp::body::json())
            .and(with_config(self.config.clone()))
            .and_then(update_config);

        let voice_test = warp::path("api")
            .and(warp::path("voice"))
            .and(warp::path("test"))
            .and(warp::post())
            .and(warp::body::json())
            .and(with_voice_service(self.voice_service.clone()))
            .and_then(test_voice);

        config_get.or(config_post).or(voice_test)
    }
}

fn with_config(config: Arc<RwLock<SystemConfig>>) -> impl Filter<Extract = (Arc<RwLock<SystemConfig>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
}

fn with_voice_service(service: Arc<VoiceProcessingService>) -> impl Filter<Extract = (Arc<VoiceProcessingService>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || service.clone())
}

async fn get_config(config: Arc<RwLock<SystemConfig>>) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    let config = config.read().await;
    Ok(warp::reply::json(&*config))
}

async fn update_config(
    new_config: SystemConfig,
    config: Arc<RwLock<SystemConfig>>
) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    let mut config_lock = config.write().await;
    *config_lock = new_config;

    // TODO: Save to file
    // config_lock.save_to_file(CONFIG_PATH)?;

    Ok(warp::reply::json(&serde_json::json!({"status": "ok"})))
}

#[derive(serde::Deserialize)]
struct VoiceTestRequest {
    text: String,
}

async fn test_voice(
    request: VoiceTestRequest,
    voice_service: Arc<VoiceProcessingService>
) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    // For now, just return success - TODO: implement actual voice testing
    tracing::info!("Voice test requested for text: {}", request.text);
    Ok(warp::reply::json(&serde_json::json!({"status": "ok", "text": request.text})))
}