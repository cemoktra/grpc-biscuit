use std::str::FromStr;

use biscuit_auth::Biscuit;
use tonic::{metadata::MetadataKey, service::Interceptor, Request, Status};

pub struct ClientInterceptor {
    header: String,
    biscuit: Biscuit,
}

impl ClientInterceptor {
    pub fn new(header: impl Into<String>, biscuit: Biscuit) -> Self {
        Self {
            header: header.into(),
            biscuit,
        }
    }
}

impl Interceptor for ClientInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let biscuit = self
            .biscuit
            .to_base64()
            .map_err(|err| Status::internal(format!("Failed to get biscuit bytes: {err}")))?;

        let key = MetadataKey::from_str(&self.header).map_err(|err| {
            Status::internal(format!("Failed to build key '{}': {err}", self.header))
        })?;
        request.metadata_mut().insert(
            key,
            biscuit.try_into().map_err(|err| {
                Status::internal(format!("Failed to convet biscuit to metadata value: {err}"))
            })?,
        );
        Ok(request)
    }
}
