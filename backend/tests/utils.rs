use backend::http::initialize_app_state;
use backend::utils::config::Config;
use envconfig::Envconfig;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub async fn setup_server() -> (String, JoinHandle<()>) {
    dotenv::dotenv().ok();

    let config = Config::init_from_env().expect("Failed to load config");
    let port = config.port;

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get local address");

    let state = initialize_app_state(config);

    let server_handle = tokio::spawn(async move {
        let app = backend::http::router(state.clone()).with_state(state);
        axum::serve(listener, app).await.expect("Server error");
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    (format!("http://{}", addr), server_handle)
}
