use biscuit_auth::{Authorizer, Biscuit};
use tonic::{Request, Status};

pub trait RequestAuthorizer {
    fn authorize(&self, authorizer: &Authorizer) -> Result<(), Status>;
}

impl<T> RequestAuthorizer for Request<T> {
    fn authorize(&self, authorizer: &Authorizer) -> Result<(), Status> {
        let biscuit: Option<&Biscuit> = self.extensions().get();

        match biscuit {
            Some(biscuit) => {
                biscuit
                    .authorize(authorizer)
                    .map_err(|_err| Status::permission_denied("permission denied"))?;
                Ok(())
            }
            None => Err(Status::unauthenticated("no biscuit found")),
        }
    }
}
