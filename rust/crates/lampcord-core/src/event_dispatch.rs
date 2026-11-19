use std::{collections::HashMap, sync::Arc};

use serde_json::Value;

pub trait GatewayEventHandler {
    fn name(&self) -> &str;
    fn supported_events(&self) -> &'static [&'static str];
    fn handle(&self, event_type: &str, data: Option<&Value>) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandlerError {
    pub handler: String,
    pub event: String,
    pub message: String,
}

#[derive(Default)]
pub struct Dispatcher {
    by_type: HashMap<String, Vec<Arc<dyn GatewayEventHandler>>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, handler: Arc<dyn GatewayEventHandler>) {
        for event in handler.supported_events() {
            self.by_type
                .entry((*event).to_owned())
                .or_default()
                .push(handler.clone());
        }
    }

    pub fn dispatch(&self, event_type: Option<&str>, data: Option<&Value>) -> Vec<HandlerError> {
        let Some(event_type) = event_type else {
            return Vec::new();
        };
        let Some(handlers) = self.by_type.get(event_type) else {
            return Vec::new();
        };

        let mut errors = Vec::new();
        for handler in handlers {
            if let Err(message) = handler.handle(event_type, data) {
                errors.push(HandlerError {
                    handler: handler.name().to_owned(),
                    event: event_type.to_owned(),
                    message,
                });
            }
        }
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHandler {
        name: &'static str,
        events: &'static [&'static str],
        result: Result<(), &'static str>,
        calls: Arc<std::sync::Mutex<Vec<String>>>,
    }

    impl GatewayEventHandler for TestHandler {
        fn name(&self) -> &str {
            self.name
        }

        fn supported_events(&self) -> &'static [&'static str] {
            self.events
        }

        fn handle(&self, event_type: &str, _data: Option<&Value>) -> Result<(), String> {
            self.calls
                .lock()
                .expect("calls lock")
                .push(format!("{}:{}", self.name, event_type));
            self.result.map_err(str::to_owned)
        }
    }

    fn handler(
        name: &'static str,
        result: Result<(), &'static str>,
        calls: &Arc<std::sync::Mutex<Vec<String>>>,
    ) -> Arc<dyn GatewayEventHandler> {
        Arc::new(TestHandler {
            name,
            events: &["MESSAGE_CREATE"],
            result,
            calls: calls.clone(),
        })
    }

    #[test]
    fn dispatch_preserves_registration_order_and_isolates_errors() {
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut dispatcher = Dispatcher::new();
        dispatcher.register(handler("message", Ok(()), &calls));
        dispatcher.register(handler("notification", Err("boom"), &calls));
        dispatcher.register(handler("voice", Ok(()), &calls));

        let errors = dispatcher.dispatch(Some("MESSAGE_CREATE"), None);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].handler, "notification");
        assert_eq!(
            *calls.lock().expect("calls lock"),
            vec![
                "message:MESSAGE_CREATE".to_owned(),
                "notification:MESSAGE_CREATE".to_owned(),
                "voice:MESSAGE_CREATE".to_owned(),
            ]
        );
    }

    #[test]
    fn dispatch_ignores_missing_type_or_handlers() {
        let dispatcher = Dispatcher::new();
        assert!(dispatcher.dispatch(None, None).is_empty());
        assert!(dispatcher.dispatch(Some("READY"), None).is_empty());
    }
}
