use rustical::{
    Args, cmd_default,
    config::{Config, DataStoreConfig, HttpConfig, SqliteDataStoreConfig},
};
use std::{
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    sync::Arc,
    thread::{self, JoinHandle},
};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

pub fn find_free_port() -> Option<u16> {
    let mut port = 15000;
    // Frees the socket on drop such that this function returns a free port
    while TcpListener::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)).is_err() {
        port += 1;

        if port >= 16000 {
            return None;
        }
    }
    Some(port)
}

pub fn rustical_process() -> (CancellationToken, u16, JoinHandle<()>, Arc<Notify>) {
    let port = find_free_port().unwrap();
    let token = CancellationToken::new();
    let cloned_token = token.clone();
    let start_notify = Arc::new(Notify::new());
    let cloned_start_notify = start_notify.clone();

    let main_process = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let fut = async {
            cmd_default(
                Args {
                    config_file: "asldajldakjsdkj".to_owned(),
                    no_migrations: false,
                    command: None,
                },
                Config {
                    data_store: DataStoreConfig::Sqlite(SqliteDataStoreConfig {
                        db_url: ":memory:".to_owned(),
                        run_repairs: true,
                        skip_broken: false,
                    }),
                    http: HttpConfig {
                        host: "127.0.0.1".to_owned(),
                        port,
                        ..Default::default()
                    },
                    frontend: Default::default(),
                    oidc: None,
                    tracing: Default::default(),
                    dav_push: Default::default(),
                    nextcloud_login: Default::default(),
                    caldav: Default::default(),
                },
                Some(cloned_start_notify),
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
