use biscuit_auth::{Authorizer, Biscuit, RootKeyProvider};
use tonic::{service::Interceptor, Request, Status};

#[derive(Clone)]
pub struct AuthorizerInterceptor<P> {
    header: String,
    authorizer: Authorizer,
    key_provider: P,
}

impl<P> AuthorizerInterceptor<P> {
    pub fn new(header: impl Into<String>, authorizer: Authorizer, key_provider: P) -> Self {
        Self {
            header: header.into(),
            authorizer,
            key_provider,
        }
    }
}

impl<P> Interceptor for AuthorizerInterceptor<P>
where
    P: RootKeyProvider + Clone,
{
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let biscuit = request
            .metadata()
            .get(&self.header)
            .ok_or(Status::unauthenticated(format!(
                "'{}' header missing",
                self.header
            )))?;
        let biscuit = Biscuit::from_base64(biscuit, self.key_provider.clone())
            .map_err(|err| Status::unauthenticated(format!("invalid biscuit: {err}")))?;
        biscuit
            .authorize(&self.authorizer)
            .map_err(|err| Status::permission_denied(format!("permission denied: {err}")))?;
        request.extensions_mut().insert(biscuit);

        Ok(request)
    }
}
