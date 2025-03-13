use backend::http::initialize_app_state;
use backend::utils::config::Config;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub async fn setup_server() -> (String, JoinHandle<()>) {
    dotenv::dotenv().ok();

    let config = Config {
        port: 0,
        database_url: "postgres://salauskilke:secret@postgresd:5432/salauskilke?sslmode=disable"
            .to_string(),
    };

    // Binding to 0 lets the os assing free port to allow multi-threaded
    // e2e tests. This avoids address already in use error.
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port))
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
