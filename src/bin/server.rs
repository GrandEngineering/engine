use bincode::serialize;
use engine::{get_auth, get_uid};
use enginelib::{
    Identifier, RawIdentier, Registry,
    api::EngineAPI,
    chrono::Utc,
    event::{debug, info, warn},
    events::{self, Events, ID},
    plugin::LibraryManager,
    task::{SolvedTasks, StoredExecutingTask, StoredTask, Task, TaskQueue},
};
use proto::{
    TaskState,
    engine_server::{Engine, EngineServer},
};
use std::{
    env::consts::OS,
    io::Read,
    sync::{Arc, RwLock as RS_RwLock},
};
use tokio::sync::RwLock;
use tonic::{Request, Status, metadata::MetadataValue, transport::Server};

mod proto {
    tonic::include_proto!("engine");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("engine_descriptor");
}
#[allow(non_snake_case)]
struct EngineService {
    pub EngineAPI: Arc<RwLock<EngineAPI>>,
}
#[tonic::async_trait]
impl Engine for EngineService {
    /// Retrieves a paginated list of tasks filtered by namespace, task name, and state.
    ///
    /// Authenticates the request and, if authorized, returns tasks in the specified state
    /// (`Processing`, `Queued`, or `Solved`) for the given namespace and task name. The results
    /// are sorted by task ID and paginated according to the requested page and page size.
    ///
    /// Returns a `TaskPage` containing the filtered tasks and pagination metadata, or a
    /// permission denied error if authentication fails.
    ///
    /// # Examples
    ///
    /// ```
    /// // Example usage within a tonic gRPC client context:
    /// let request = proto::TaskPageRequest {
    ///     namespace: "example_ns".to_string(),
    ///     task: "example_task".to_string(),
    ///     state: proto::TaskState::Queued as i32,
    ///     page: 0,
    ///     page_size: 10,
    /// };
    /// let response = engine_client.get_tasks(request).await?;
    /// assert!(response.get_ref().tasks.len() <= 10);
    /// ```
    async fn get_tasks(
        &self,
        request: tonic::Request<proto::TaskPageRequest>,
    ) -> std::result::Result<tonic::Response<proto::TaskPage>, tonic::Status> {
        let mut api = self.EngineAPI.write().await;
        let challenge = get_auth(&request);
        let uid = get_uid(&request);
        let db = api.db.clone();
        if !Events::CheckAuth(&mut api, uid, challenge, db) {
            //TODO: change to AdminSpecific Auth
            info!("GetTask denied due to Invalid Auth");
            return Err(Status::permission_denied("Invalid authentication"));
        };
        let data = request.get_ref();

        let q: Vec<proto::Task> = match data.clone().state() {
            TaskState::Processing => {
                match api
                    .executing_tasks
                    .tasks
                    .get(&(data.namespace.clone(), data.task.clone()))
                {
                    Some(tasks) => {
                        let mut task_refs: Vec<_> = tasks.iter().collect();
                        task_refs.sort_by_key(|f| &f.id);
                        task_refs
                            .iter()
                            .map(|f| proto::Task {
                                id: f.id.clone(),
                                task_id: format!("{}:{}", data.namespace, data.task),
                                task_payload: f.bytes.clone(),
                                payload: Vec::new(),
                            })
                            .collect()
                    }
                    None => {
                        info!(
                            "Namespace {:?} and task {:?} not found in Processing state",
                            data.namespace, data.task
                        );
                        Vec::new()
                    }
                }
            }
            TaskState::Queued => {
                match api
                    .task_queue
                    .tasks
                    .get(&(data.namespace.clone(), data.task.clone()))
                {
                    Some(tasks) => {
                        let mut d = tasks.clone();
                        d.sort_by_key(|f| f.id.clone());
                        d.iter()
                            .map(|f| proto::Task {
                                id: f.id.clone(),
                                task_id: format!("{}:{}", data.namespace, data.task),
                                task_payload: f.bytes.clone(),
                                payload: Vec::new(),
                            })
                            .collect()
                    }
                    None => {
                        info!(
                            "Namespace {:?} and task {:?} not found in Queued state",
                            data.namespace, data.task
                        );
                        Vec::new()
                    }
                }
            }
            TaskState::Solved => {
                match api
                    .solved_tasks
                    .tasks
                    .get(&(data.namespace.clone(), data.task.clone()))
                {
                    Some(tasks) => {
                        let mut d = tasks.clone();
                        d.sort_by_key(|f| f.id.clone());
                        d.iter()
                            .map(|f| proto::Task {
                                id: f.id.clone(),
                                task_id: format!("{}:{}", data.namespace, data.task),
                                task_payload: f.bytes.clone(),
                                payload: Vec::new(),
                            })
                            .collect()
                    }
                    None => {
                        info!(
                            "Namespace {:?} and task {:?} not found in Solved state",
                            data.namespace, data.task
                        );
                        Vec::new()
                    }
                }
            }
        };
        let index = data.page * data.page_size as u64;
        let end = index + data.page_size as u64;
        let final_vec: Vec<_> = q
            .iter()
            .skip(index as usize)
            .take(data.page_size as usize)
            .cloned()
            .collect();
        return Ok(tonic::Response::new(proto::TaskPage {
            namespace: data.namespace.clone(),
            task: data.task.clone(),
            page: data.page,
            page_size: data.page_size,
            state: data.state,
            tasks: final_vec,
        }));
    }
    /// Handles custom gRPC messages with admin-level authentication.
    ///
    /// Processes a CGRPC request by verifying admin credentials and dispatching the event payload to the appropriate handler. Returns the processed event payload in the response. If authentication fails, returns a permission denied error.
    ///
    /// # Returns
    /// A `Cgrpcmsg` response containing the processed event payload, or a permission denied gRPC status on failed authentication.
    ///
    /// # Examples
    ///
    /// ```
    /// // Example usage within a gRPC client context:
    /// let request = proto::Cgrpcmsg {
    ///     handler_mod_id: "mod".to_string(),
    ///     handler_id: "handler".to_string(),
    ///     event_payload: vec![1, 2, 3],
    ///     // ... other fields ...
    /// };
    /// let response = engine_service.cgrpc(tonic::Request::new(request)).await?;
    /// assert_eq!(response.get_ref().handler_mod_id, "mod");
    /// ```
    async fn cgrpc(
        &self,
        request: tonic::Request<proto::Cgrpcmsg>,
    ) -> std::result::Result<tonic::Response<proto::Cgrpcmsg>, tonic::Status> {
        info!(
            "CGRPC request received for handler: {}:{}",
            request.get_ref().handler_mod_id,
            request.get_ref().handler_id
        );
        let mut api = self.EngineAPI.write().await;
        let challenge = get_auth(&request);
        let db = api.db.clone();
        debug!("Checking admin authentication for CGRPC request");
        let output = Events::CheckAdminAuth(
            &mut api,
            challenge,
            (
                request.get_ref().handler_mod_id.clone(),
                request.get_ref().handler_id.clone(),
            ),
            db,
        );
        if !output {
            warn!("CGRPC auth check failed - permission denied");
            return Err(tonic::Status::permission_denied("Invalid CGRPC Auth"));
        };
        let out = Arc::new(std::sync::RwLock::new(Vec::new()));
        debug!("Dispatching CGRPC event to handler");
        Events::CgrpcEvent(
            &mut api,
            ID("engine_core", "grpc"),
            request.get_ref().event_payload.clone(),
            out.clone(),
        );
        let mut res = request.get_ref().clone();
        res.event_payload = out.read().unwrap().clone();
        info!("CGRPC request processed successfully");
        return Ok(tonic::Response::new(res));
    }
    async fn aquire_task_reg(
        &self,
        request: tonic::Request<proto::Empty>,
    ) -> Result<tonic::Response<proto::TaskRegistry>, tonic::Status> {
        let uid = get_uid(&request);
        let challenge = get_auth(&request);
        info!("Task registry request received from user: {}", uid);
        let mut api = self.EngineAPI.write().await;
        let db = api.db.clone();

        debug!("Validating authentication for task registry request");
        if !Events::CheckAuth(&mut api, uid.clone(), challenge, db) {
            info!(
                "Task registry request denied - invalid authentication for user: {}",
                uid
            );
            return Err(Status::permission_denied("Invalid authentication"));
        };
        let mut tasks: Vec<RawIdentier> = Vec::new();
        for (k, v) in &api.task_registry.tasks {
            let js: Vec<String> = vec![k.0.clone(), k.1.clone()];
            let jstr = js.join(":");
            tasks.push(jstr);
        }
        info!("Returning task registry with {} tasks", tasks.len());
        let response = proto::TaskRegistry { tasks };
        Ok(tonic::Response::new(response))
    }

    async fn aquire_task(
        &self,
        request: tonic::Request<proto::TaskRequest>,
    ) -> Result<tonic::Response<proto::Task>, tonic::Status> {
        let challenge = get_auth(&request);
        let input = request.get_ref();
        let task_id = input.task_id.clone();
        let uid = get_uid(&request);
        info!(
            "Task acquisition request received from user: {} for task: {}",
            uid, task_id
        );

        let mut api = self.EngineAPI.write().await;
        let db = api.db.clone();
        debug!("Validating authentication for task acquisition");
        if !Events::CheckAuth(&mut api, uid.clone(), challenge, db) {
            info!(
                "Task acquisition denied - invalid authentication for user: {}",
                uid
            );
            return Err(Status::permission_denied("Invalid authentication"));
        };

        // Todo: check for wrong input to not cause a Panic out of bounds.
        let alen = &task_id.split(":").collect::<Vec<&str>>().len();
        if *alen != 2 {
            info!("Invalid task ID format: {}", task_id);
            return Err(Status::invalid_argument(
                "Invalid task ID format, expected 'namespace:name",
            ));
        }
        let namespace = &task_id.split(":").collect::<Vec<&str>>()[0];
        let task_name = &task_id.split(":").collect::<Vec<&str>>()[1];
        debug!("Looking up task definition for {}:{}", namespace, task_name);
        let tsx = api
            .task_registry
            .get(&(namespace.to_string(), task_name.to_string()));
        if tsx.is_none() {
            warn!(
                "Task acquisition failed - task does not exist: {}:{}",
                namespace, task_name
            );
            return Err(Status::invalid_argument("Task Does not Exist"));
        }
        let mut map = api
            .task_queue
            .tasks
            .get(&ID(namespace, task_name))
            .unwrap()
            .clone();
        let ttask = map.first().unwrap().clone();
        let task_payload = ttask.bytes.clone();
        map.remove(0);
        // Get Task and remove it from queue
        api.task_queue.tasks.insert(ID(namespace, task_name), map);
        let store = bincode::serialize(&api.task_queue.clone()).unwrap();
        api.db.insert("tasks", store).unwrap();
        // Move it to exec queue
        let mut exec_tsks = api
            .executing_tasks
            .tasks
            .get(&ID(namespace, task_name))
            .unwrap()
            .clone();
        exec_tsks.push(enginelib::task::StoredExecutingTask {
            bytes: task_payload.clone(),
            user_id: uid.clone(),
            given_at: Utc::now(),
            id: ttask.id.clone(),
        });
        api.executing_tasks
            .tasks
            .insert(ID(namespace, task_name), exec_tsks);
        let store = bincode::serialize(&api.executing_tasks.clone()).unwrap();
        api.db.insert("executing_tasks", store).unwrap();
        let response = proto::Task {
            id: ttask.id,
            task_id: input.task_id.clone(),
            task_payload,
            payload: Vec::new(),
        };
        Ok(tonic::Response::new(response))
    }
    async fn publish_task(
        &self,
        request: tonic::Request<proto::Task>,
    ) -> Result<tonic::Response<proto::Empty>, tonic::Status> {
        let mut api = self.EngineAPI.write().await;
        let challenge = get_auth(&request);
        let uid = get_uid(&request);
        let db = api.db.clone();

        let task_id = request.get_ref().task_id.clone();
        let alen = &task_id.split(":").collect::<Vec<&str>>().len();
        if *alen != 2 {
            return Err(Status::invalid_argument("Invalid Params"));
        }
        let namespace = &task_id.split(":").collect::<Vec<&str>>()[0];
        let task_name = &task_id.split(":").collect::<Vec<&str>>()[1];

        if !Events::CheckAuth(&mut api, uid.clone(), challenge, db) {
            info!("Aquire Task denied due to Invalid Auth");
            return Err(Status::permission_denied("Invalid authentication"));
        };
        if !api
            .task_registry
            .tasks
            .contains_key(&ID(namespace, task_name))
        {
            warn!(
                "Task acquisition failed - task does not exist: {}:{}",
                namespace, task_name
            );
            return Err(Status::invalid_argument("Task Does not Exist"));
        }
        let mem_tsk = api
            .executing_tasks
            .tasks
            .get(&ID(namespace, task_name))
            .unwrap()
            .clone();
        let tsk = mem_tsk
            .iter()
            .find(|f| f.id == task_id.clone() && f.user_id == uid.clone());
        if let Some(tsk) = tsk {
            let reg_tsk = api
                .task_registry
                .get(&ID(namespace, task_name))
                .unwrap()
                .clone();
            if !reg_tsk.verify(request.get_ref().task_payload.clone()) {
                info!("Failed to parse task");
                return Err(Status::invalid_argument("Failed to parse given task bytes"));
            }
            // Exec Tasks -> DB
            let mut nmem_tsk = mem_tsk.clone();
            nmem_tsk.retain(|f| f.id != task_id.clone() && f.user_id != uid.clone());
            api.executing_tasks
                .tasks
                .insert(ID(namespace, task_name), nmem_tsk.clone());
            let t_mem_execs = api.executing_tasks.clone();
            api.db
                .insert("executing_tasks", bincode::serialize(&t_mem_execs).unwrap())
                .unwrap();
            // tsk-> solved Tsks
            let mut mem_solv = api
                .solved_tasks
                .tasks
                .get(&ID(namespace, task_name))
                .unwrap()
                .clone();
            mem_solv.push(enginelib::task::StoredTask {
                bytes: tsk.bytes.clone(),
                id: tsk.id.clone(),
            });
            api.solved_tasks
                .tasks
                .insert(ID(namespace, task_name), mem_solv);
            // Solved tsks -> DB
            let e_solv = bincode::serialize(&api.solved_tasks.tasks).unwrap();
            api.db.insert("solved_tasks", e_solv).unwrap();
            info!("Task published successfully: {} by user: {}", task_id, uid);
            return Ok(tonic::Response::new(proto::Empty {}));
        } else {
            return Err(tonic::Status::not_found("Invalid taskid or userid"));
        }
    }
    async fn create_task(
        &self,
        request: tonic::Request<proto::Task>,
    ) -> Result<tonic::Response<proto::Task>, tonic::Status> {
        let mut api = self.EngineAPI.write().await;
        let challenge = get_auth(&request);
        let uid = get_uid(&request);
        let db = api.db.clone();
        if !Events::CheckAuth(&mut api, uid, challenge, db) {
            //TODO: change to AdminSpecific Auth
            info!("Create Task denied due to Invalid Auth");
            return Err(Status::permission_denied("Invalid authentication"));
        };
        let task = request.get_ref();
        let task_id = task.task_id.clone();
        let id: Identifier = (
            task_id.split(":").collect::<Vec<&str>>()[0].to_string(),
            task_id.split(":").collect::<Vec<&str>>()[1].to_string(),
        );
        let tsk_reg = api.task_registry.get(&id);
        if let Some(tsk_reg) = tsk_reg {
            if !tsk_reg.clone().verify(task.task_payload.clone()) {
                warn!("Failed to parse given task bytes");
                return Err(Status::invalid_argument("Failed to parse given task bytes"));
            }
            let tbp_tsk = StoredTask {
                bytes: task.task_payload.clone(),
                id: druid::Druid::default().to_hex(),
            };
            let mut mem_tsks = api.task_queue.clone();
            let mut mem_tsk = mem_tsks.tasks.get(&id).unwrap().clone();
            mem_tsk.push(tbp_tsk.clone());
            mem_tsks.tasks.insert(id.clone(), mem_tsk);
            api.task_queue = mem_tsks;
            api.db
                .insert(
                    "tasks",
                    bincode::serialize(&api.task_queue.clone()).unwrap(),
                )
                .unwrap();
            return Ok(tonic::Response::new(proto::Task {
                id: tbp_tsk.id.clone(),
                task_id: task_id.clone(),
                payload: Vec::new(),
                task_payload: tbp_tsk.bytes.clone(),
            }));
        }
        Err(tonic::Status::aborted("Error"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut api = EngineAPI::default();
    EngineAPI::init(&mut api);
    Events::init_auth(&mut api);
    Events::StartEvent(&mut api);
    let addr = api.cfg.config_toml.port.parse().unwrap();
    let apii = Arc::new(RwLock::new(api));
    EngineAPI::init_chron(apii.clone());
    let engine = EngineService { EngineAPI: apii };

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build_v1alpha()
        .unwrap();

    Server::builder()
        .add_service(reflection_service)
        .add_service(EngineServer::new(engine))
        .serve(addr)
        .await?;

    Ok(())
}
