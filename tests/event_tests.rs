use enginelib::{
    RegisterEventHandler, Registry,
    api::EngineAPI,
    event::{
        EngineEventHandlerRegistry, EngineEventRegistry, Event, EventBus, EventCTX, EventHandler,
    },
    events::ID,
    plugin::LibraryManager,
    task::{SolvedTasks, TaskQueue, Verifiable},
};
use macros::Verifiable;
use sled::Config;
use std::{any::Any, collections::HashMap, sync::Arc};
use tracing_test::traced_test;

#[traced_test]
#[test]
fn id() {
    assert!(ID("namespace", "id") == ID("namespace", "id"))
}

#[traced_test]
#[test]
fn test_event_registration_and_handling() {
    let mut api = EngineAPI::test_default();

    // Create a test event
    #[derive(Clone, Debug)]
    struct TestEvent {
        pub value: i32,
        pub cancelled: bool,
        pub id: (String, String),
    }

    impl Event for TestEvent {
        fn clone_box(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }
        fn cancel(&mut self) {
            self.cancelled = true;
        }
        fn is_cancelled(&self) -> bool {
            self.cancelled
        }
        fn get_id(&self) -> (String, String) {
            self.id.clone()
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    // Create a test handler
    RegisterEventHandler!(TestHandler, TestEvent, |event: &mut TestEvent| {
        event.value += 1;
    });

    // Register event and handler
    let event_id = ID("test", "test_event");
    api.event_bus.event_registry.register(
        Arc::new(TestEvent {
            value: 0,
            cancelled: false,
            id: event_id.clone(),
        }),
        event_id.clone(),
    );

    api.event_bus
        .event_handler_registry
        .register_handler(TestHandler, event_id.clone());

    // Test event handling
    let mut test_event = TestEvent {
        value: 0,
        cancelled: false,
        id: event_id.clone(),
    };

    api.event_bus.handle(event_id, &mut test_event);
    assert_eq!(test_event.value, 1);
    drop(api.db);
}

#[traced_test]
#[test]
fn test_task_registration() {
    use enginelib::task::Task;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, Verifiable)]
    struct TestTask {
        pub value: i32,
        pub id: (String, String),
    }

    impl Task for TestTask {
        fn get_id(&self) -> (String, String) {
            self.id.clone()
        }
        fn clone_box(&self) -> Box<dyn Task> {
            Box::new(self.clone())
        }
        fn run_cpu(&mut self) {
            self.value += 1;
        }
        fn to_bytes(&self) -> Vec<u8> {
            bincode::serialize(self).unwrap()
        }
        fn from_bytes(&self, bytes: &[u8]) -> Box<dyn Task> {
            Box::new(bincode::deserialize::<TestTask>(bytes).unwrap())
        }
        fn from_toml(&self, d: String) -> Box<dyn Task> {
            return Box::new(self.clone());
        }
    }
    let mut api = EngineAPI::test_default();
    let task_id = ID("test", "test_task");

    // Register the task type
    api.task_registry.register(
        Arc::new(TestTask {
            value: 0,
            id: task_id.clone(),
        }),
        task_id.clone(),
    );

    // Verify it was registered
    assert!(api.task_registry.tasks.contains_key(&task_id));
    drop(api.db);
}

#[traced_test]
#[test]
fn test_task_execution() {
    use enginelib::task::{Runner, Task};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, Verifiable)]
    struct TestTask {
        pub value: i32,
        pub id: (String, String),
    }

    impl Task for TestTask {
        fn from_toml(&self, d: String) -> Box<dyn Task> {
            return Box::new(self.clone());
        }
        fn get_id(&self) -> (String, String) {
            self.id.clone()
        }
        fn clone_box(&self) -> Box<dyn Task> {
            Box::new(self.clone())
        }
        fn run_cpu(&mut self) {
            self.value += 1;
        }
        fn to_bytes(&self) -> Vec<u8> {
            bincode::serialize(self).unwrap()
        }
        fn from_bytes(&self, bytes: &[u8]) -> Box<dyn Task> {
            Box::new(bincode::deserialize::<TestTask>(bytes).unwrap())
        }
    }

    // Test task execution directly
    let mut task = TestTask {
        value: 0,
        id: ID("test", "test_task"),
    };

    task.run(Some(Runner::CPU));
    assert_eq!(task.value, 1);
}

#[traced_test]
#[test]
fn test_task_serialization() {
    use bincode;
    use enginelib::task::{StoredTask, Task};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, Verifiable)]
    struct TestTask {
        pub value: i32,
        pub id: (String, String),
    }

    impl Task for TestTask {
        fn from_toml(&self, d: String) -> Box<dyn Task> {
            return Box::new(self.clone());
        }
        fn get_id(&self) -> (String, String) {
            self.id.clone()
        }
        fn clone_box(&self) -> Box<dyn Task> {
            Box::new(self.clone())
        }
        fn run_cpu(&mut self) {
            self.value += 1;
        }
        fn to_bytes(&self) -> Vec<u8> {
            bincode::serialize(self).unwrap()
        }
        fn from_bytes(&self, bytes: &[u8]) -> Box<dyn Task> {
            Box::new(bincode::deserialize::<TestTask>(bytes).unwrap())
        }
    }

    let task = TestTask {
        value: 42,
        id: ID("test", "test_task"),
    };

    // Test serialization and deserialization
    let serialized = task.to_bytes();
    let stored_task = StoredTask {
        bytes: serialized,
        id: "id".into(),
    };

    // Deserialize
    let deserialized_task: TestTask = bincode::deserialize(&stored_task.bytes).unwrap();
    assert_eq!(deserialized_task.value, 42);

    // Test the from_bytes function
    let recreated_task = task.from_bytes(&stored_task.bytes);
    // We need a way to check the value inside the recreated task
    // Since we can't directly access the value, we'll serialize it again and deserialize manually
    let bytes = recreated_task.to_bytes();
    let final_task: TestTask = bincode::deserialize(&bytes).unwrap();
    assert_eq!(final_task.value, 42);
}
