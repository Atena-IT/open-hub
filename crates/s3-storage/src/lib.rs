use aws_config::{BehaviorVersion, Region};
use aws_credential_types::Credentials;
use aws_sdk_s3::{
    config::Builder as S3ConfigBuilder,
    presigning::PresigningConfig,
    primitives::ByteStream,
    Client,
};
use std::time::Duration;

#[derive(Clone)]
pub struct S3Client {
    inner:          Client,
    presign_client: Client,
    bucket:         String,
}

impl S3Client {
    /// Create an S3 client.  Pass `endpoint_url = Some(...)` to target MinIO.
    /// `public_endpoint_url` is used for presigned URLs visible to external clients.
    pub async fn new(
        endpoint_url: Option<&str>,
        region: &str,
        bucket: &str,
        access_key: &str,
        secret_key: &str,
    ) -> anyhow::Result<Self> {
        Self::with_public_endpoint(endpoint_url, None, region, bucket, access_key, secret_key).await
    }

    /// Create an S3 client with an optional separate public endpoint for presigning.
    pub async fn with_public_endpoint(
        endpoint_url: Option<&str>,
        public_endpoint_url: Option<&str>,
        region: &str,
        bucket: &str,
        access_key: &str,
        secret_key: &str,
    ) -> anyhow::Result<Self> {
        let creds = Credentials::new(access_key, secret_key, None, None, "static");

        let mut builder = S3ConfigBuilder::new()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(region.to_owned()))
            .credentials_provider(creds.clone())
            .force_path_style(true);

        if let Some(ep) = endpoint_url {
            builder = builder.endpoint_url(ep);
        }

        let inner = Client::from_conf(builder.build());

        // Build a separate presigning client if public endpoint differs
        let public_ep = public_endpoint_url.or(endpoint_url);
        let presign_client = if public_ep != endpoint_url {
            let mut pb = S3ConfigBuilder::new()
                .behavior_version(BehaviorVersion::latest())
                .region(Region::new(region.to_owned()))
                .credentials_provider(creds)
                .force_path_style(true);
            if let Some(ep) = public_ep {
                pb = pb.endpoint_url(ep);
            }
            Client::from_conf(pb.build())
        } else {
            inner.clone()
        };

        Ok(Self { inner, presign_client, bucket: bucket.to_owned() })
    }

    /// Upload bytes directly (single-part, suitable for objects ≤ 5 GiB).
    pub async fn put_object(&self, key: &str, data: bytes::Bytes) -> anyhow::Result<()> {
        let len = data.len() as i64;
        self.inner
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data))
            .content_length(len)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("S3 put_object failed: {e}"))?;
        Ok(())
    }

    /// Check whether an object exists (HEAD request).
    pub async fn object_exists(&self, key: &str) -> anyhow::Result<bool> {
        match self.inner
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_)  => Ok(true),
            Err(e) => {
                if e.as_service_error()
                    .map(|se| se.is_not_found())
                    .unwrap_or(false)
                {
                    Ok(false)
                } else {
                    Err(anyhow::anyhow!("S3 head_object failed: {e}"))
                }
            }
        }
    }

    /// Generate a pre-signed GET URL valid for `expiry`.
    pub async fn presign_get(
        &self,
        key: &str,
        expiry: Duration,
    ) -> anyhow::Result<String> {
        let cfg = PresigningConfig::expires_in(expiry)
            .map_err(|e| anyhow::anyhow!("presign config error: {e}"))?;

        let presigned = self.presign_client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(cfg)
            .await
            .map_err(|e| anyhow::anyhow!("presign failed: {e}"))?;

        Ok(presigned.uri().to_string())
    }

    /// Generate a pre-signed PUT URL valid for `expiry`.
    pub async fn presign_put(
        &self,
        key: &str,
        expiry: Duration,
    ) -> anyhow::Result<String> {
        let cfg = PresigningConfig::expires_in(expiry)
            .map_err(|e| anyhow::anyhow!("presign config error: {e}"))?;

        let presigned = self.presign_client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(cfg)
            .await
            .map_err(|e| anyhow::anyhow!("presign PUT failed: {e}"))?;

        Ok(presigned.uri().to_string())
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    /// Download an object's bytes from S3.
    pub async fn get_object(&self, key: &str) -> anyhow::Result<bytes::Bytes> {
        let resp = self.inner
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("S3 get_object failed: {e}"))?;

        let data = resp.body.collect().await
            .map_err(|e| anyhow::anyhow!("S3 body read failed: {e}"))?;

        Ok(data.into_bytes())
    }
}

/// Build the S3 key for a Xorb (two-level prefix from first 4 hex chars).
pub fn xorb_key(hex_hash: &str) -> String {
    format!("xorbs/{}/{}/{}", &hex_hash[..2], &hex_hash[2..4], hex_hash)
}

/// Build the S3 key for a Shard.
pub fn shard_key(hex_hash: &str) -> String {
    format!("shards/{}/{}", &hex_hash[..2], hex_hash)
}

/// Build the S3 key for an LFS object by SHA-256 OID.
pub fn lfs_key(oid: &str) -> String {
    format!("lfs/{}/{}/{}", &oid[..2], &oid[2..4], oid)
}

/// Build the S3 key for a repo file upload.
pub fn repo_file_key(repo_full_name: &str, sha256: &str) -> String {
    format!("files/{}/{}", repo_full_name, sha256)
}
