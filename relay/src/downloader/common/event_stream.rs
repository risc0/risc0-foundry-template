use ethers::{
    self,
    providers::{JsonRpcClient, Middleware, Provider, PubsubClient},
};
use tokio_stream::Stream;

// This wrapper only exists to provide a nicer API and play well with the rest
// of the components. TODO: Evaluate if it's actually necessary
pub struct EventStream<P: JsonRpcClient> {
    filter: ethers::types::Filter,
    provider: Provider<P>,
}

impl<P: JsonRpcClient> EventStream<P> {
    pub fn new(provider: Provider<P>, address: ethers::types::Address) -> Self {
        Self {
            provider,
            filter: ethers::types::Filter::new().address(address),
        }
    }

    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.filter = self.filter.event(name.as_ref());
        self
    }

    pub fn from_block(mut self, block: u64) -> Self {
        self.filter = self.filter.from_block(block);
        self
    }
}

impl<P: JsonRpcClient> EventStream<P> {
    pub async fn poll(&self) -> impl Stream<Item = ethers::types::Log> + '_ {
        self.provider.watch(&self.filter).await.unwrap().stream()
    }
}

impl<P: PubsubClient> EventStream<P> {
    pub async fn watch(&self) -> impl Stream<Item = ethers::types::Log> + '_ {
        self.provider.subscribe_logs(&self.filter).await.unwrap()
    }
}
