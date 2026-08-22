use rustical::{
    Args, cmd_serve,
    config::{Config, DataStoreConfig, HttpConfig, PostgresDataStoreConfig, SqliteDataStoreConfig},
};
use std::{
    collections::HashSet,
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    sync::{Arc, Mutex, OnceLock},
    thread::{self, JoinHandle},
};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

// When running multiple integration tests we need to make sure that they don't get the same port
static BOUND_PORTS: OnceLock<Mutex<HashSet<u16>>> = OnceLock::new();

pub fn find_free_port() -> Option<u16> {
    let bound_ports = BOUND_PORTS.get_or_init(Mutex::default);
    let mut bound_ports_write = bound_ports.lock().unwrap();
    let mut port = 15000;
    // Frees the socket on drop such that this function returns a free port
    while TcpListener::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)).is_err()
        || bound_ports_write.contains(&port)
    {
        port += 1;

        if port >= 16000 {
            return None;
        }
    }
    bound_ports_write.insert(port);
    Some(port)
}

pub fn sqlite_store(db_url: Option<String>) -> DataStoreConfig {
    DataStoreConfig::Sqlite(SqliteDataStoreConfig {
        db_url: db_url.unwrap_or_else(|| ":memory:".to_owned()),
        run_repairs: true,
        skip_broken: false,
    })
}

pub async fn postgres_store() -> Option<DataStoreConfig> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let admin = sqlx::PgPool::connect(&url).await.expect("connect postgres");
    let schema = format!("t_{}", uuid::Uuid::new_v4().simple());
    sqlx::query(sqlx::AssertSqlSafe(format!("CREATE SCHEMA {schema}")))
        .execute(&admin)
        .await
        .expect("create test schema");
    let separator = if url.contains('?') { '&' } else { '?' };
    Some(DataStoreConfig::Postgres(PostgresDataStoreConfig {
        db_url: format!("{url}{separator}options=-c%20search_path%3D{schema}"),
        run_repairs: true,
        skip_broken: false,
    }))
}

pub fn rustical_process(
    data_store: DataStoreConfig,
) -> (CancellationToken, u16, JoinHandle<()>, Arc<Notify>) {
    let port = find_free_port().unwrap();
    let token = CancellationToken::new();
    let cloned_token = token.clone();
    let start_notify = Arc::new(Notify::new());
    let cloned_start_notify = start_notify.clone();

    let main_process = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let fut = async {
            cmd_serve(
                Args {
                    config_file: "asldajldakjsdkj".to_owned(),
                    no_migrations: false,
                    command: rustical::Command::Serve,
                },
                Config {
                    data_store,
                    http: HttpConfig {
                        bind: Some(format!("127.0.0.1:{port}")),
                        ..Default::default()
                    },
                    frontend: Default::default(),
                    oidc: None,
                    tracing: Default::default(),
                    dav_push: Default::default(),
                    nextcloud_login: Default::default(),
                    caldav: Default::default(),
                    maintenance: Default::default(),
                },
                Some(cloned_start_notify),
                false,
            )
            .await
        };
        rt.block_on(async {
            tokio::select! {
                _ = cloned_token.cancelled() => {},
                _ = fut => {}
            }
        });
    });
    (token, port, main_process, start_notify)
}
