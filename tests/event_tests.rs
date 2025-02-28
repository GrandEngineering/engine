use enginelib::{
    RegisterEventHandler, Registry,
    api::EngineAPI,
    event::{Event, EventCTX, EventHandler},
    events::ID,
};
use std::sync::Arc;
use tracing_test::traced_test;

#[traced_test]
#[test]
fn id() {
    assert!(ID("namespace", "id") == ID("namespace", "id"))
}

#[traced_test]
#[test]
fn test_event_registration_and_handling() {
    let mut api = EngineAPI::default();

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
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
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
}

#[traced_test]
#[test]
fn test_task_queue() {
    use enginelib::task::{Runner, Task};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
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
    }

    let mut api = EngineAPI::default();
    let task = TestTask {
        value: 0,
        id: ID("test", "test_task"),
    };

    api.task_queue.tasks.push(Box::new(task.clone()));

    // Test task execution
    if let Some(task) = api.task_queue.tasks.first_mut() {
        let task: &mut TestTask = unsafe { &mut *((task as *mut Box<dyn Task>) as *mut TestTask) };
        let mut task = task.clone();
        task.run(Some(Runner::CPU));
        assert_eq!(task.value, 1);
    }
}

#[traced_test]
#[test]
fn test_task_serialization() {
    use enginelib::task::{Task, TaskQueueStorage};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
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
    }

    let mut api = EngineAPI::default();
    let task = TestTask {
        value: 42,
        id: ID("test", "test_task"),
    };

    api.task_queue.tasks.push(Box::new(task));

    // Test serialization
    let storage = TaskQueueStorage::from_task_queue(&api.task_queue);
    assert_eq!(storage.tasks.len(), 1);

    // Register task type
    api.task_registry.register(
        Arc::new(TestTask {
            value: 0,
            id: ID("test", "test_task"),
        }),
        ID("test", "test_task"),
    );

    // Test deserialization
    let new_queue = enginelib::task::TaskQueue::from_storage(&storage, &api);
    assert_eq!(new_queue.tasks.len(), 1);
}
