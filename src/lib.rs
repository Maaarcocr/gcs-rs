use std::sync::Arc;

use grpcio::{ChannelBuilder, ChannelCredentials, EnvBuilder};

mod protos;

#[derive(Clone)]
pub struct GcsClient {
    client: protos::storage_grpc::StorageClient,
}

const URL: &str = "storage.googleapis.com";

impl GcsClient {
    pub fn new() -> Result<Self, anyhow::Error> {
        let env = Arc::new(EnvBuilder::new().build());
        let credentials = ChannelCredentials::google_default_credentials()?;
        let channel = ChannelBuilder::new(env)
            .set_credentials(credentials)
            .connect(URL);
        Ok(Self {
            client: protos::storage_grpc::StorageClient::new(channel),
        })
    }

    pub fn list_objects(
        &mut self,
        bucket: &str,
        prefix: &str,
    ) -> grpcio::Result<Vec<protos::storage::Object>> {
        let request = protos::storage::ListObjectsRequest {
            parent: bucket.to_string(),
            prefix: prefix.to_string(),
            ..Default::default()
        };

        let response = self.client.list_objects(&request)?;
        Ok(response.objects.to_vec())
    }

    // pub fn list_objects(
    //     &mut self,
    //     bucket: &str,
    //     prefix: &str,
    // ) -> Result<Vec<Object>, tonic::Status> {
    //     let request = tonic::Request::new(ListObjectsRequest {
    //         parent: bucket.to_string(),
    //         prefix: prefix.to_string(),
    //         ..Default::default()
    //     });

    //     let request = self.add_auth_to_request(request);

    //     let response = self.client.list_objects(request).await?;
    //     Ok(response.into_inner().objects)
    // }

    // pub async fn read_object(
    //     &mut self,
    //     bucket: &str,
    //     object: &str,
    // ) -> Result<impl AsyncRead, tonic::Status> {
    //     let request = tonic::Request::new(google::storage::v2::ReadObjectRequest {
    //         bucket: bucket.to_string(),
    //         object: object.to_string(),
    //         ..Default::default()
    //     });

    //     let request = self.add_auth_to_request(request);

    //     let response = self.client.read_object(request).await?;
    //     let response = response.into_inner();
    //     Ok(StreamReader::new(response.filter_map(
    //         |chunk| match chunk {
    //             Ok(chunk) => match chunk.checksummed_data {
    //                 Some(data) => Some(Ok(std::io::Cursor::new(data.content))),
    //                 None => None,
    //             },
    //             Err(e) => Some(Err(std::io::Error::from(GrpcError::from(e)))),
    //         },
    //     )))
    // }

    // fn add_auth_to_request<T>(&self, mut req: Request<T>) -> Request<T> {
    //     req.metadata_mut()
    //         .insert("Authorization", self.token.as_str().parse().unwrap());
    //     req
    // }
}
