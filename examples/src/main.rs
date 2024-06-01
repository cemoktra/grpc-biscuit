pub mod example {
    tonic::include_proto!("example");
}

use std::net::SocketAddr;

use biscuit_auth::{
    error::Format,
    macros::{authorizer, biscuit},
    KeyPair, PublicKey, RootKeyProvider,
};
use example::{
    example_client::ExampleClient,
    example_server::{Example, ExampleServer},
    AuthenticatedEchoRequest, AuthenticatedEchoResponse,
};
use grpc_biscuit::{AuthorizerInterceptor, ClientInterceptor, RequestAuthorizer};
use tonic::{
    async_trait,
    transport::{Endpoint, Server},
    Request, Response, Status,
};

#[derive(Default)]
pub struct ExampleService {}

#[async_trait]
impl Example for ExampleService {
    async fn authenticated_echo(
        &self,
        request: Request<AuthenticatedEchoRequest>,
    ) -> Result<Response<AuthenticatedEchoResponse>, Status> {
        let operation = "write";
        let authorizer = authorizer!(r#"operation({operation});"#);
        request
            .authorize(&authorizer)
            .map_err(|err| Status::permission_denied(format!("Permission denied: {err}")))?;
        Ok(Response::new(AuthenticatedEchoResponse {
            message: request.into_inner().message,
        }))
    }
}

#[derive(Clone)]
struct KeyProvider {
    public_key: PublicKey,
}

impl KeyProvider {
    pub fn new(public_key: PublicKey) -> Self {
        Self { public_key }
    }
}

impl RootKeyProvider for KeyProvider {
    fn choose(&self, _key_id: Option<u32>) -> Result<PublicKey, Format> {
        Ok(self.public_key)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key_pair = KeyPair::new();
    let key_provider = KeyProvider::new(key_pair.public());
    let header = "authorization";
    let addr = "[::1]:50051".parse()?;
    let endpoint = Endpoint::from_static("http://[::1]:50051");

    let authority = biscuit!(
        r#"
        operation("read");
        resource("echo");
        right("1234","echo","read");
        user("1234");
        "#,
    );
    let token = authority.build(&key_pair)?;

    let authorizer = authorizer!(
        r#"
        allow if
        user($user_id),
        resource($res),
        operation($op),
        right($user_id, $res, $op);
    "#
    );

    let server_interceptor = AuthorizerInterceptor::new(header, authorizer, key_provider);
    let client_interceptor = ClientInterceptor::new(header, token);

    let _handle = tokio::task::spawn(server(addr, server_interceptor));

    std::thread::sleep(std::time::Duration::from_secs(1));

    match client(endpoint, client_interceptor).await {
        Ok(_) => panic!("This is expected to fail"),
        Err(_) => Ok(()),
    }
}

async fn server<P>(
    addr: SocketAddr,
    interceptor: AuthorizerInterceptor<P>,
) -> Result<(), tonic::transport::Error>
where
    P: RootKeyProvider + Clone + Send + Sync + 'static,
{
    Server::builder()
        .add_service(ExampleServer::with_interceptor(
            ExampleService::default(),
            interceptor,
        ))
        .serve(addr)
        .await?;

    Ok(())
}

async fn client(
    endpoint: Endpoint,
    interceptor: ClientInterceptor,
) -> Result<(), Box<dyn std::error::Error>> {
    let channel = endpoint.connect().await?;
    let mut client = ExampleClient::with_interceptor(channel, interceptor);

    let request = Request::new(AuthenticatedEchoRequest {
        message: "Hello".into(),
    });
    client.authenticated_echo(request).await?;

    Ok(())
}
