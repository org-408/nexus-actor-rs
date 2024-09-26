// This file is @generated by prost-build.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RemoteMessage {
    #[prost(oneof = "remote_message::MessageType", tags = "1, 2, 3, 4")]
    pub message_type: ::core::option::Option<remote_message::MessageType>,
}
/// Nested message and enum types in `RemoteMessage`.
pub mod remote_message {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum MessageType {
        #[prost(message, tag = "1")]
        MessageBatch(super::MessageBatch),
        #[prost(message, tag = "2")]
        ConnectRequest(super::ConnectRequest),
        #[prost(message, tag = "3")]
        ConnectResponse(super::ConnectResponse),
        #[prost(message, tag = "4")]
        DisconnectRequest(super::DisconnectRequest),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MessageBatch {
    #[prost(string, repeated, tag = "1")]
    pub type_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "2")]
    pub targets: ::prost::alloc::vec::Vec<super::actor::Pid>,
    #[prost(message, repeated, tag = "3")]
    pub envelopes: ::prost::alloc::vec::Vec<MessageEnvelope>,
    #[prost(message, repeated, tag = "4")]
    pub senders: ::prost::alloc::vec::Vec<super::actor::Pid>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MessageEnvelope {
    #[prost(int32, tag = "1")]
    pub type_id: i32,
    #[prost(bytes = "vec", tag = "2")]
    pub message_data: ::prost::alloc::vec::Vec<u8>,
    #[prost(int32, tag = "3")]
    pub target: i32,
    #[prost(int32, tag = "4")]
    pub sender: i32,
    #[prost(uint32, tag = "5")]
    pub serializer_id: u32,
    #[prost(message, optional, tag = "6")]
    pub message_header: ::core::option::Option<MessageHeader>,
    #[prost(uint32, tag = "7")]
    pub target_request_id: u32,
    #[prost(uint32, tag = "8")]
    pub sender_request_id: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MessageHeader {
    #[prost(map = "string, string", tag = "1")]
    pub header_data: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        ::prost::alloc::string::String,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActorPidRequest {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub kind: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActorPidResponse {
    #[prost(message, optional, tag = "1")]
    pub pid: ::core::option::Option<super::actor::Pid>,
    #[prost(int32, tag = "2")]
    pub status_code: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectRequest {
    #[prost(oneof = "connect_request::ConnectionType", tags = "1, 2")]
    pub connection_type: ::core::option::Option<connect_request::ConnectionType>,
}
/// Nested message and enum types in `ConnectRequest`.
pub mod connect_request {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum ConnectionType {
        #[prost(message, tag = "1")]
        ClientConnection(super::ClientConnection),
        #[prost(message, tag = "2")]
        ServerConnection(super::ServerConnection),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct DisconnectRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientConnection {
    #[prost(string, tag = "1")]
    pub system_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ServerConnection {
    #[prost(string, tag = "1")]
    pub system_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub address: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectResponse {
    #[prost(string, tag = "2")]
    pub member_id: ::prost::alloc::string::String,
    #[prost(bool, tag = "3")]
    pub blocked: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListProcessesRequest {
    #[prost(string, tag = "1")]
    pub pattern: ::prost::alloc::string::String,
    #[prost(enumeration = "ListProcessesMatchType", tag = "2")]
    pub r#type: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListProcessesResponse {
    #[prost(message, repeated, tag = "1")]
    pub pids: ::prost::alloc::vec::Vec<super::actor::Pid>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetProcessDiagnosticsRequest {
    #[prost(message, optional, tag = "1")]
    pub pid: ::core::option::Option<super::actor::Pid>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetProcessDiagnosticsResponse {
    #[prost(string, tag = "1")]
    pub diagnostics_string: ::prost::alloc::string::String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ListProcessesMatchType {
    MatchPartOfString = 0,
    MatchExactString = 1,
    MatchRegex = 2,
}
impl ListProcessesMatchType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ListProcessesMatchType::MatchPartOfString => "MatchPartOfString",
            ListProcessesMatchType::MatchExactString => "MatchExactString",
            ListProcessesMatchType::MatchRegex => "MatchRegex",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "MatchPartOfString" => Some(Self::MatchPartOfString),
            "MatchExactString" => Some(Self::MatchExactString),
            "MatchRegex" => Some(Self::MatchRegex),
            _ => None,
        }
    }
}
/// Generated client implementations.
pub mod remoting_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct RemotingClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl RemotingClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> RemotingClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> RemotingClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
        {
            RemotingClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn receive(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::RemoteMessage>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::RemoteMessage>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/remote.Remoting/Receive");
            let mut req = request.into_streaming_request();
            req.extensions_mut().insert(GrpcMethod::new("remote.Remoting", "Receive"));
            self.inner.streaming(req, path, codec).await
        }
        pub async fn list_processes(
            &mut self,
            request: impl tonic::IntoRequest<super::ListProcessesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListProcessesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/remote.Remoting/ListProcesses",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("remote.Remoting", "ListProcesses"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_process_diagnostics(
            &mut self,
            request: impl tonic::IntoRequest<super::GetProcessDiagnosticsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetProcessDiagnosticsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/remote.Remoting/GetProcessDiagnostics",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("remote.Remoting", "GetProcessDiagnostics"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod remoting_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with RemotingServer.
    #[async_trait]
    pub trait Remoting: std::marker::Send + std::marker::Sync + 'static {
        /// Server streaming response type for the Receive method.
        type ReceiveStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::RemoteMessage, tonic::Status>,
            >
            + std::marker::Send
            + 'static;
        async fn receive(
            &self,
            request: tonic::Request<tonic::Streaming<super::RemoteMessage>>,
        ) -> std::result::Result<tonic::Response<Self::ReceiveStream>, tonic::Status>;
        async fn list_processes(
            &self,
            request: tonic::Request<super::ListProcessesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListProcessesResponse>,
            tonic::Status,
        >;
        async fn get_process_diagnostics(
            &self,
            request: tonic::Request<super::GetProcessDiagnosticsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetProcessDiagnosticsResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct RemotingServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> RemotingServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for RemotingServer<T>
    where
        T: Remoting,
        B: Body + std::marker::Send + 'static,
        B::Error: Into<StdError> + std::marker::Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            match req.uri().path() {
                "/remote.Remoting/Receive" => {
                    #[allow(non_camel_case_types)]
                    struct ReceiveSvc<T: Remoting>(pub Arc<T>);
                    impl<
                        T: Remoting,
                    > tonic::server::StreamingService<super::RemoteMessage>
                    for ReceiveSvc<T> {
                        type Response = super::RemoteMessage;
                        type ResponseStream = T::ReceiveStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                tonic::Streaming<super::RemoteMessage>,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Remoting>::receive(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ReceiveSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/remote.Remoting/ListProcesses" => {
                    #[allow(non_camel_case_types)]
                    struct ListProcessesSvc<T: Remoting>(pub Arc<T>);
                    impl<
                        T: Remoting,
                    > tonic::server::UnaryService<super::ListProcessesRequest>
                    for ListProcessesSvc<T> {
                        type Response = super::ListProcessesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListProcessesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Remoting>::list_processes(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ListProcessesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/remote.Remoting/GetProcessDiagnostics" => {
                    #[allow(non_camel_case_types)]
                    struct GetProcessDiagnosticsSvc<T: Remoting>(pub Arc<T>);
                    impl<
                        T: Remoting,
                    > tonic::server::UnaryService<super::GetProcessDiagnosticsRequest>
                    for GetProcessDiagnosticsSvc<T> {
                        type Response = super::GetProcessDiagnosticsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetProcessDiagnosticsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Remoting>::get_process_diagnostics(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetProcessDiagnosticsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", tonic::Code::Unimplemented as i32)
                                .header(
                                    http::header::CONTENT_TYPE,
                                    tonic::metadata::GRPC_CONTENT_TYPE,
                                )
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T> Clone for RemotingServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    /// Generated gRPC service name
    pub const SERVICE_NAME: &str = "remote.Remoting";
    impl<T> tonic::server::NamedService for RemotingServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
