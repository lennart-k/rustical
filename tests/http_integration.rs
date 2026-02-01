// This integration test checks whether the HTTP server works by actually running rustical in a new
// thread.
use common::rustical_process;
use headers::{HeaderMapExt, Location};
use http::{Method, StatusCode};
use reqwest::{
    Url,
    cookie::{CookieStore, Jar},
    redirect::Policy,
};
use rustical::{
    PrincipalsArgs, cmd_health, cmd_principals,
    config::{Config, DataStoreConfig, HttpConfig, SqliteDataStoreConfig},
    principals::{CreateArgs, EditArgs, PrincipalsCommand},
};
use rustical_store::auth::{AuthenticationProvider, PrincipalType};
use rustical_store_sqlite::{create_db_pool, principal_store::SqlitePrincipalStore};
use std::{collections::HashMap, time::Duration};

mod common;

pub async fn test_runner<O, F>(db_path: Option<String>, inner: F)
where
    O: IntoFuture<Output = ()>,
    // <O as IntoFuture>::IntoFuture: UnwindSafe,
    F: FnOnce(u16) -> O,
{
    // Start RustiCal process
    let (token, port, main_process, start_notify) = rustical_process(db_path);

    // Wait for RustiCal server to listen
    tokio::time::timeout(Duration::new(2, 0), start_notify.notified())
        .await
        .unwrap();

    // We use catch_unwind to make sure we'll always correctly stop RustiCal
    // Otherwise, our process would just run indefinitely
    inner(port).into_future().await;

    // Signal RustiCal to stop
    token.cancel();
    main_process.join().unwrap();
}

#[tokio::test]
async fn test_ping() {
    test_runner(None, async |port| {
        let origin = format!("http://localhost:{port}");
        let resp = reqwest::get(origin.clone() + "/ping").await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Ensure that path normalisation works as intended
        let resp = reqwest::get(origin + "/ping/").await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        cmd_health(
            HttpConfig {
                host: "localhost".to_owned(),
                port,
                ..Default::default()
            },
            Default::default(),
        )
        .await
        .unwrap();
    })
    .await
}

// When setting a use password from the CLI we effectively have two processes accessing the same
// database: The server and the CLI.
// This test ensures that the server correctly picks up the changes made by the CLI.
#[tokio::test]
async fn test_initial_setup() {
    let db_tempfile = tempfile::NamedTempFile::with_suffix(".rustical-test.sqlite3").unwrap();
    let db_path = db_tempfile.path().to_string_lossy().into_owned();

    test_runner(Some(db_path.clone()), async |port| {
        let origin = format!("http://localhost:{port}");
        // Create principal
        cmd_principals(
            PrincipalsArgs {
                command: PrincipalsCommand::Create(CreateArgs {
                    id: "user".to_owned(),
                    name: Some("Test User".to_owned()),
                    password: false,
                    for_testing_password_from_arg: None,
                    principal_type: Some(PrincipalType::Individual),
                }),
            },
            Config {
                data_store: DataStoreConfig::Sqlite(SqliteDataStoreConfig {
                    db_url: db_path.clone(),
                    run_repairs: true,
                    skip_broken: false,
                }),
                http: Default::default(),
                frontend: Default::default(),
                oidc: None,
                tracing: Default::default(),
                dav_push: Default::default(),
                nextcloud_login: Default::default(),
                caldav: Default::default(),
            },
        )
        .await
        .unwrap();
        // Set principal password
        cmd_principals(
            PrincipalsArgs {
                command: PrincipalsCommand::Edit(EditArgs {
                    id: "user".to_owned(),
                    name: None,
                    password: false,
                    remove_password: false,
                    for_testing_password_from_arg: Some("pass".to_owned()),
                    principal_type: Some(PrincipalType::Individual),
                }),
            },
            Config {
                data_store: DataStoreConfig::Sqlite(SqliteDataStoreConfig {
                    db_url: db_path.clone(),
                    run_repairs: true,
                    skip_broken: false,
                }),
                http: Default::default(),
                frontend: Default::default(),
                oidc: None,
                tracing: Default::default(),
                dav_push: Default::default(),
                nextcloud_login: Default::default(),
                caldav: Default::default(),
            },
        )
        .await
        .unwrap();

        let client = reqwest::Client::builder()
            .redirect(Policy::none())
            .cookie_store(true)
            .build()
            .unwrap();
        {
            // Log in to the frontend
            let url = origin.clone() + "/frontend/login";
            let mut form = HashMap::new();
            form.insert("username", "user");
            form.insert("password", "pass");
            let resp = client
                .request(Method::POST, &url)
                .form(&form)
                .send()
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::SEE_OTHER);
            let location = resp.headers().get("Location").unwrap().to_str().unwrap();
            assert_eq!(location, "/frontend/user");
        }

        {
            let url = origin.clone() + "/frontend/user";
            let resp = client.request(Method::GET, &url).send().await.unwrap();
            assert_eq!(resp.status(), StatusCode::SEE_OTHER);
            let location = resp.headers().get("Location").unwrap().to_str().unwrap();
            assert_eq!(location, "/frontend/user/user");
        }

        {
            let url = origin.clone() + "/frontend/user/user";
            let resp = client.request(Method::GET, &url).send().await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }

        let app_token = {
            let url = origin.clone() + "/frontend/user/user/app_token";
            let mut form = HashMap::new();
            form.insert("name", "Test Token");
            let resp = client
                .request(Method::POST, &url)
                .form(&form)
                .send()
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::OK);

            resp.text().await.unwrap()
        };

        let url = origin.clone() + "/caldav/principal/user";
        let resp = reqwest::Client::new()
            .request(Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let resp = reqwest::Client::new()
            .request(Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .basic_auth("user", Some(&app_token))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::MULTI_STATUS);

        let db = create_db_pool(&db_path, false).await.unwrap();
        let principal_store = SqlitePrincipalStore::new(db);
        principal_store.remove_principal("user").await.unwrap();

        let resp = reqwest::Client::new()
            .request(Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    })
    .await;
}
