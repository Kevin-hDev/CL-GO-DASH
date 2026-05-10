use std::collections::HashMap;
use std::sync::Arc;

use super::channels::ChannelAdapter;
use super::types::ChannelKey;

pub struct GatewayRegistry {
    adapters: HashMap<ChannelKey, Arc<dyn ChannelAdapter>>,
}

impl GatewayRegistry {
    pub fn new() -> Self {
        Self { adapters: HashMap::new() }
    }

    pub fn register(&mut self, key: ChannelKey, adapter: Arc<dyn ChannelAdapter>) {
        self.adapters.insert(key, adapter);
    }

    pub fn get(&self, key: &ChannelKey) -> Option<&Arc<dyn ChannelAdapter>> {
        self.adapters.get(key)
    }

    pub fn remove(&mut self, key: &ChannelKey) -> Option<Arc<dyn ChannelAdapter>> {
        self.adapters.remove(key)
    }

    pub fn list_active(&self) -> Vec<&ChannelKey> {
        self.adapters.keys().collect()
    }

    pub fn list_by_channel(&self, channel_id: &str) -> Vec<(&ChannelKey, &Arc<dyn ChannelAdapter>)> {
        self.adapters
            .iter()
            .filter(|(k, _)| k.channel_id == channel_id)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.adapters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::gateway::channels::{
        capabilities::ChannelCapabilities, ChannelAdapter, ChannelContext,
        DeliveryReceipt, GatewayResult, InboundMessage, OutboundMessage,
    };
    use async_trait::async_trait;
    use crate::models::ChannelAccountConfig;

    struct MockAdapter;

    #[async_trait]
    impl ChannelAdapter for MockAdapter {
        fn id(&self) -> &'static str { "mock" }
        fn capabilities(&self) -> ChannelCapabilities {
            ChannelCapabilities::telegram()
        }
        async fn validate_config(&self, _: &ChannelAccountConfig) -> GatewayResult<()> { Ok(()) }
        async fn start(&self, _: ChannelContext, _: tokio::sync::mpsc::Sender<InboundMessage>) -> GatewayResult<tokio::task::JoinHandle<()>> {
            Ok(tokio::spawn(async {}))
        }
        async fn send(&self, _: OutboundMessage) -> GatewayResult<DeliveryReceipt> {
            Ok(DeliveryReceipt { message_id: "1".into() })
        }
        async fn health(&self) -> bool { true }
    }

    #[test]
    fn register_and_get() {
        let mut reg = GatewayRegistry::new();
        let key = ChannelKey::new("telegram", "bot1");
        reg.register(key.clone(), Arc::new(MockAdapter));
        assert!(reg.get(&key).is_some());
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn list_by_channel_filters() {
        let mut reg = GatewayRegistry::new();
        reg.register(ChannelKey::new("telegram", "bot1"), Arc::new(MockAdapter));
        reg.register(ChannelKey::new("telegram", "bot2"), Arc::new(MockAdapter));
        reg.register(ChannelKey::new("slack", "work"), Arc::new(MockAdapter));

        assert_eq!(reg.list_by_channel("telegram").len(), 2);
        assert_eq!(reg.list_by_channel("slack").len(), 1);
        assert_eq!(reg.list_by_channel("discord").len(), 0);
    }

    #[test]
    fn remove_adapter() {
        let mut reg = GatewayRegistry::new();
        let key = ChannelKey::new("telegram", "bot1");
        reg.register(key.clone(), Arc::new(MockAdapter));
        assert!(reg.remove(&key).is_some());
        assert!(reg.is_empty());
    }
}
