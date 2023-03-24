use gcp_auth::AuthenticationManager;
use gcp_auth::Token;
use google::storage::v2::storage_client::StorageClient;
use google::storage::v2::ListObjectsRequest;
use google::storage::v2::Object;
use tokio::io::AsyncRead;
use tokio_stream::StreamExt;
use tokio_util::io::StreamReader;
use tonic::transport::Channel;
use tonic::transport::Endpoint;
use tonic::Request;

pub mod google {
    pub mod storage {
        pub mod v2 {
            tonic::include_proto!("google.storage.v2");
        }
    }
    pub mod iam {
        pub mod v1 {
            tonic::include_proto!("google.iam.v1");
        }
    }

    pub mod r#type {
        tonic::include_proto!("google.r#type");
    }
}

#[derive(Clone)]
pub struct GcsClient {
    client: StorageClient<Channel>,
    token: Token,
}

struct GrpcError {
    code: tonic::Code,
}

impl From<tonic::Status> for GrpcError {
    fn from(status: tonic::Status) -> Self {
        Self {
            code: status.code(),
        }
    }
}

impl From<GrpcError> for std::io::Error {
    fn from(error: GrpcError) -> Self {
        match error.code {
            tonic::Code::NotFound => std::io::ErrorKind::NotFound,
            tonic::Code::PermissionDenied => std::io::ErrorKind::PermissionDenied,
            _ => std::io::ErrorKind::Other,
        }
        .into()
    }
}


impl GcsClient {
    pub async fn new<T: Into<Endpoint>>(endpoint: T) -> Result<Self, anyhow::Error> {
        let authentication_manager = AuthenticationManager::new().await?;
        let scopes = &["https://www.googleapis.com/auth/devstorage.read_only"];
        let token = authentication_manager.get_token(scopes).await?;
        Ok(Self {
            client: StorageClient::connect(endpoint).await?,
            token,
        })
    }

    pub async fn list_objects(
        &mut self,
        bucket: &str,
        prefix: &str,
    ) -> Result<Vec<Object>, tonic::Status> {
        let request = tonic::Request::new(ListObjectsRequest {
            parent: bucket.to_string(),
            prefix: prefix.to_string(),
            ..Default::default()
        });

        let request = self.add_auth_to_request(request);

        let response = self.client.list_objects(request).await?;
        Ok(response.into_inner().objects)
    }

    pub async fn read_object(
        &mut self,
        bucket: &str,
        object: &str,
    ) -> Result<impl AsyncRead, tonic::Status> {
        let request = tonic::Request::new(google::storage::v2::ReadObjectRequest {
            bucket: bucket.to_string(),
            object: object.to_string(),
            ..Default::default()
        });

        let request = self.add_auth_to_request(request);

        let response = self.client.read_object(request).await?;
        let response = response.into_inner();
        Ok(StreamReader::new(response.filter_map(
            |chunk| match chunk {
                Ok(chunk) => match chunk.checksummed_data {
                    Some(data) => Some(Ok(std::io::Cursor::new(data.content))),
                    None => None,
                },
                Err(e) => Some(Err(std::io::Error::from(GrpcError::from(e)))),
            },
        )))
    }

    fn add_auth_to_request<T>(&self, mut req: Request<T>) -> Request<T> {
        req.metadata_mut()
            .insert("Authorization", self.token.as_str().parse().unwrap());
        req
    }
}
