pub use bincode;
pub use jail;
pub use memmap;
pub use mockito;
pub use nix;
pub use procedural_macros::*;
use serde::Deserialize;
pub use serde_yaml;

#[derive(Deserialize)]
pub struct MockDefinition {
    pub request: MockRequest,
    pub response: MockResponse,
}

#[derive(Deserialize)]
pub struct MockRequest {
    pub method: String,
    pub path: Option<String>,
    pub headers: Option<Vec<MockHeader>>,
}

#[derive(Deserialize)]
pub struct MockHeader {
    pub header: String,
    pub value: String,
}

#[derive(Deserialize)]
pub struct MockResponse {
    pub headers: Option<Vec<MockHeader>>,
    pub body: Option<String>,
}

#[macro_export]
macro_rules! fixture {
    ($file:expr) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test/resources/",
            $file
        ))
    };
}

#[macro_export]
macro_rules! bytes_fixture {
    ($file:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test/resources/",
            $file
        ))
    };
}

#[macro_export]
macro_rules! code_fixture {
    ($file:expr) => {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test/resources/",
            $file
        ))
    };
}

#[macro_export]
macro_rules! fixture_path {
    ($file:expr) => {
        std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test/resources/",
            $file
        ))
    };
}

/// Generate mockito mocks using declarative (yml) definition.
#[macro_export]
macro_rules! mock_server {
    ($file:expr) => {{
        use $crate::mockito::{mock, Matcher};
        use $crate::serde_yaml;
        use $crate::*;

        let mocks_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test/resources/server_mocks",
        );

        let file = std::fs::read(format!("{}/{}", mocks_path, $file))
            .expect("definition file's not found!");

        let definitions: Vec<MockDefinition> = serde_yaml::from_slice(&file)
            .expect("failed to parse definition file");

        let mocks = definitions
            .into_iter()
            .map(|MockDefinition { request, response }| {
                let path =
                    request.path.map(Matcher::Regex).unwrap_or(Matcher::Any);

                let mut mock = mock(&request.method, path);

                if let Some(headers) = request.headers {
                    for MockHeader { header, value } in headers {
                        mock = mock.match_header(&header, &value[..]);
                    }
                }

                if let Some(body) = response.body {
                    mock = mock.with_body_from_file(format!(
                        "{}/{}",
                        mocks_path, body
                    ));
                }

                if let Some(headers) = response.headers {
                    for MockHeader { header, value } in headers {
                        let value = value
                            .replace("SERVER_URL", &mockito::server_url());

                        mock = mock.with_header(&header, &value[..]);
                    }
                }

                mock.create()
            })
            .collect::<Vec<_>>();

        (mockito::server_url(), mocks)
    }};
}
