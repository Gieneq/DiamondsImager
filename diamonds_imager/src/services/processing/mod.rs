/// Module for managing work dispatching and worker execution.
/// Handles queuing work, assigning it to workers, and collecting results.
pub mod worker;
pub mod image_manip;

use std::{
    collections::HashMap, 
    sync::Arc, 
    time::Duration
};

use worker::{
    Work, 
    WorkId, 
    WorkResult, 
    WorkResultWrapped, 
    WorkWrapped, 
    Worker
};

const WORK_ORDERS_QUEUE_CAP: usize = 32;
const WORKERS_COUNT: usize = 2;
const WORKERS_RESULTS_QUEUE_CAP: usize = WORK_ORDERS_QUEUE_CAP;

/// Errors that can occur during processing work.
#[derive(Debug, thiserror::Error)]
pub enum ProcessingError {
    #[error("Busy")]
    Busy,

    #[error("Service failed")]
    ServiceFailed,

    #[error("Not available")]
    NotAvailable,
}

/// Internal structure representing an ordered work to be processed.
#[derive(Debug)]
struct WorkOrder {
    work: Work,
    assign_id_tx: tokio::sync::oneshot::Sender<WorkId>,
}

/// Dispatcher responsible for managing work submission and result collection.
#[derive(Debug)]
pub struct WorkDispatcher {
    processings_queue_tx: tokio::sync::mpsc::Sender<WorkOrder>,
    dipatcher_task: tokio::task::JoinHandle<()>,
    orders_results: Arc<tokio::sync::Mutex<HashMap<WorkId, WorkResult>>>,
    on_result_notify: Arc<tokio::sync::Notify>,
}

impl WorkDispatcher {
    /// Collects the result from a worker and stores it.
    async fn process_collected_result(
        work_result: WorkResultWrapped,
        orders_results_shared: &Arc<tokio::sync::Mutex<HashMap<WorkId, WorkResult>>>
    ) {
        let mut orders_results_shared_guard = orders_results_shared.lock().await;
        debug_assert!(!orders_results_shared_guard.contains_key(&work_result.id), "Keys should be unique!");
        orders_results_shared_guard.insert(work_result.id, work_result.work_result);
    }

    /// Assigns a work order to an available worker.
    async fn assign_order_to_worker(
        work_order: WorkOrder,
        next_unique_work_id: &mut u64,
        workers: &[Worker],
        assigning_start_idx: usize
    ) {
        // Assign a new unique work ID
        let new_id = {
            let tmp_next_id = *next_unique_work_id;
            *next_unique_work_id += 1;
            tmp_next_id
        };

        // Inform sender, work was received and id assigned
        // Err means sender no longer is interested in this offer
        if let Err(_) = work_order.assign_id_tx.send(new_id) {
            // Discard recently assigned id and continue
            return;
        }

        // Sender got notified, probably will poll for finished work with assigned id
        // Let one of the workers do work
        let mut work_wrapped = WorkWrapped {
            work: work_order.work,
            id: new_id
        };
        
        // Attempt to pass WorkOrder to one of workers
        loop {
            for worker_idx in (0..WORKERS_COUNT).map(|i| (assigning_start_idx + i) % WORKERS_COUNT) {
                let worker = &workers[worker_idx];
                if let Err(e) = worker.try_enque_work(work_wrapped) {
                    work_wrapped = match e {
                        tokio::sync::mpsc::error::TrySendError::Full(work_wrapped_not_enqueued) => work_wrapped_not_enqueued,
                        tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                            panic!("Worker should not be closed before dispatcher")
                        },
                    }
                } else {
                    tracing::info!("Work {new_id} was assigned to worker {worker}");
                    return;
                }
            }

            // All workers busy, try again laater
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }

    /// Creates a new work dispatcher with configured workers.
    pub fn new() -> Self {
        let (work_orders_queue_tx, mut work_orders_queue_rx) = tokio::sync::mpsc::channel::<WorkOrder>(WORK_ORDERS_QUEUE_CAP);
        
        // Orders collector
        let orders_results = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
        let orders_results_shared = orders_results.clone();

        let on_result_notify = Arc::new(tokio::sync::Notify::new());
        let on_result_notify_shared = on_result_notify.clone();

        let dipatcher_task = tokio::task::spawn(async move {
            // Unique work_id
            let mut next_unique_work_id = 0;

            // Method to assing work to tasks, for sure can be done better
            // LATER: track free workers
            let mut assigning_fuzz_factor = 0;

            // Channels to collect results from multiple workers
            let (work_result_tx, mut work_result_rx) = tokio::sync::mpsc::channel(WORKERS_RESULTS_QUEUE_CAP);

            {
                // Spawn workers
                let workers = (0..WORKERS_COUNT).map(|id| Worker::new(id as u32, work_result_tx.clone())).collect::<Vec<_>>();
                
                // Enter dispatcher loop
                loop {
                    tokio::select! {
                        work_order = work_orders_queue_rx.recv() => match work_order {
                            Some(work_order) => {
                                Self::assign_order_to_worker(work_order, &mut next_unique_work_id, &workers, assigning_fuzz_factor).await;
                                assigning_fuzz_factor += 1;
                                assigning_fuzz_factor %= WORKERS_COUNT;
                            },
                            None => {
                                tracing::info!("Stopping work dispatcher, workers will got stopped soon");
                                break;
                            }
                        },
                        work_result = work_result_rx.recv() => match work_result {
                            Some(work_result) => {
                                Self::process_collected_result(work_result, &orders_results_shared).await;
                                on_result_notify_shared.notify_waiters();
                            },
                            None => {
                                tracing::warn!("Unexpected workers closing!");
                                break;
                            }
                        }
                    }
                }

                // Shutdown workers
                for worker in workers {
                    worker.shutdown().await;
                }

            }
        });

        Self { 
            processings_queue_tx: work_orders_queue_tx, 
            dipatcher_task,
            orders_results,
            on_result_notify
        }
    }

    /// Enqueues a work item and returns the assigned work ID.
    pub async fn enque_work(&self, work_to_do: Work) -> Result<WorkId, ProcessingError> {
        let (assign_id_tx, assign_id_rx) = tokio::sync::oneshot::channel();
        let work_order = WorkOrder { work: work_to_do, assign_id_tx };

        // Enque work order
        let result = tokio::time::timeout(Duration::from_millis(250), async {
            self.processings_queue_tx.send(work_order).await
        }).await;

        // Work queue can be full
        let _ = match result {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(_)) => Err(ProcessingError::ServiceFailed),
            Err(_) => Err(ProcessingError::Busy),
        }?;

        // Await assigned id
        assign_id_rx.await
            .map_err(|_| ProcessingError::ServiceFailed)
    }

    /// Tries to retrieve the result for a given work ID.
    /// If a timeout is specified, waits for the result up to the given duration.
    /// Awaiting results is event driven by receiving notification from dispatcher task.
    pub async fn get_work_result(&self, work_id: WorkId, timeout_duration: Option<Duration>) -> Result<WorkResult, ProcessingError> {
        if let Some(timeout) = timeout_duration {
            let deadline = tokio::time::Instant::now() + timeout;

            loop {
                {
                    let mut orders_results_shared_guard = self.orders_results.lock().await;
                    if let Some(work_result) = orders_results_shared_guard.remove(&work_id) {
                        return Ok(work_result);
                    }
                }

                let now = tokio::time::Instant::now();

                if now >= deadline {
                    return Err(ProcessingError::NotAvailable); // timeout hit
                }

                let remaining_time = deadline - now;
        
                if let Err(_) = tokio::time::timeout(remaining_time, self.on_result_notify.notified()).await {
                    return Err(ProcessingError::NotAvailable);
                }
            }
        } else {
            // Instant try
            let mut orders_results_shared_guard = self.orders_results.lock().await;
            orders_results_shared_guard.remove(&work_id).ok_or(ProcessingError::NotAvailable)
        }
    }

    /// Shuts down the work dispatcher and waits for all tasks to complete.
    pub async fn shutdown(self) {
        drop(self.processings_queue_tx);
        self.dipatcher_task.await.expect("Dispatcher should be shutdown gracefully");
        tracing::info!("Work dispatcher shutdown!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::dmc::PaletteDmc;
    use std::time::Duration;

    fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_test_writer()
            .try_init();
    }

    #[tokio::test]
    async fn test_dropping_dispatcher_should_shutdown_workers() {
        init_tracing();
        
        let dispatcher = WorkDispatcher::new();
        drop(dispatcher);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    #[tokio::test]
    async fn test_dispatcher_shutdown_should_shutdown_workers_first() {
        init_tracing();

        let dispatcher = WorkDispatcher::new();

        // No work enqueued.
        dispatcher.shutdown().await;

        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    #[tokio::test]
    async fn test_dispatcher_simple_enqueue_and_result() {
        init_tracing();

        let dispatcher = WorkDispatcher::new();

        let palette_dmc = Arc::new(PaletteDmc::load_from_file_default().unwrap());
        let src_image = Arc::new(ditherum::image_utils::generate_gradient_image(
            100,
            20,
            image::Rgb([0,0,0]),
            image::Rgb([255,0,0]),
        ));

        let work = Work::PaletteExtract { 
            palette_dmc, 
            src_image, 
            max_colors: Some(5)
        };

        let work_id = dispatcher.enque_work(work).await.expect("Failed to enqueue work");

        let result = dispatcher.get_work_result(work_id, Some(Duration::from_millis(1000))).await.expect("Should has work result by now");
        match result {
            WorkResult::PaletteExtract { dmc_bom } => {
                assert!(!dmc_bom.is_empty(), "DMC BOM should not be empty");
            },
            _ => panic!("Unexpected work result variant"),
        }

        dispatcher.shutdown().await;
    }

    #[tokio::test]
    async fn test_dispatcher_multiple_works() {
        init_tracing();

        let dispatcher = WorkDispatcher::new();
        let mut work_ids = vec![];

        for _ in 0..5 {
            let work = Work::TestWork { delay: Duration::from_millis(100) };

            let work_id = dispatcher.enque_work(work).await.expect("Failed to enqueue work");
            work_ids.push(work_id);
        }

        for work_id in work_ids {
            let result = dispatcher.get_work_result(work_id, Some(Duration::from_millis(1000))).await.expect("Should has work result by now");
            assert!(matches!(result, WorkResult::TestWork), "Unexpected work result variant {result:?}");
        }

        dispatcher.shutdown().await;
    }

    #[tokio::test]
    async fn test_dispatcher_work_not_ready_yet() {
        init_tracing();

        let dispatcher = WorkDispatcher::new();

        let work_id = dispatcher.enque_work(Work::TestWork { delay: Duration::from_millis(30) }).await.expect("Failed to enqueue work");
        
        let work_await_result = dispatcher.get_work_result(work_id, Some(Duration::from_millis(10))).await;
        assert!(matches!(work_await_result, Err(ProcessingError::NotAvailable)));
        
        let work_await_result = dispatcher.get_work_result(work_id, Some(Duration::from_millis(50))).await;
        assert!(matches!(work_await_result, Ok(WorkResult::TestWork)));
        
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}