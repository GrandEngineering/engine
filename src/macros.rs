#[macro_export]
macro_rules! register_event {
    ($api:expr,$mod_id:ident,$name:ident,$default_state:expr) => {{
        use $crate::Registry;
        let $name = ID(stringify!(mod_id), stringify!($name));
        $api.event_bus
            .event_registry
            .register(Arc::new($default_state), $name.clone());
    }};
}

#[macro_export]
macro_rules! RegisterEventHandler {
    ($handler:ident,$event:ty,$mod_ctx:ty, $handler_fn:expr) => {
        use std::sync::Arc;
        pub struct $handler {
            mod_ctx: Arc<$mod_ctx>,
        };
        impl $handler {
            pub fn new(mod_ctx: Arc<$mod_ctx>) -> Self {
                Self { mod_ctx }
            }
        }
        impl EventHandler for $handler {
            fn handle(&self, event: &mut dyn Event) {
                let event: &mut $event = <Self as EventCTX<$event>>::get_event::<$event>(event);
                self.handleCTX(event);
            }
        }
        impl EventCTX<$event> for $handler {
            fn handleCTX(&self, event: &mut $event) {
                let mod_ctx: &Arc<$mod_ctx> = &self.mod_ctx;
                $handler_fn(event, mod_ctx)
            }
        }
    };
    ($handler:ident,$event:ty, $handler_fn:expr) => {
        pub struct $handler;
        impl EventHandler for $handler {
            fn handle(&self, event: &mut dyn Event) {
                let event: &mut $event = <Self as EventCTX<$event>>::get_event::<$event>(event);
                self.handleCTX(event);
            }
        }
        impl EventCTX<$event> for $handler {
            fn handleCTX(&self, event: &mut $event) {
                $handler_fn(event)
            }
        }
    };
}
