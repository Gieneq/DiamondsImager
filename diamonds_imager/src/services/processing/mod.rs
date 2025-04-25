pub mod worker;

use std::{
    sync::Arc, 
    time::Duration
};

use worker::Work;

const PROCESSINGS_QUEUE_CAP: usize = 8;
const WORKERS_COUNT: usize = 2;

#[derive(Debug, thiserror::Error)]
pub enum ProcessingError {
    #[error("Busy")]
    Busy,

    #[error("ServiceFailed")]
    ServiceFailed,
}

#[derive(Debug)]
pub struct ProcessingRunner {
    processings_queue_tx: tokio::sync::mpsc::Sender<Work>,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    dipatcher_task: tokio::task::JoinHandle<()>,
}

impl ProcessingRunner {
    pub fn new() -> Self {
        let (processings_queue_tx, processings_queue_rx) = tokio::sync::mpsc::channel(PROCESSINGS_QUEUE_CAP);
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

        let dipatcher_task = tokio::task::spawn(async move {
            // let mut workers = Vec::with_capacity(WORKERS_COUNT);

            // Spawn workser
            // for _ in 0..Self::WORKERS_COUNT {
    
            //     workers_tasks.push(Worker::new());
            // }

            // Enter dispatcher loop
            // loop {
            //     if let Some(incomming_processing) = processings_queue_rx.recv().await {

            //     }
            // }

        });

        Self { 
            processings_queue_tx, 
            shutdown_tx, 
            dipatcher_task
        }
    }

    pub async fn enque_processing(&self, new_processing: Work) -> Result<(), ProcessingError> {
        let result = tokio::time::timeout(Duration::from_millis(250), async {
            self.processings_queue_tx.send(new_processing).await
        }).await;

        match result {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(_)) => Err(ProcessingError::ServiceFailed),
            Err(_) => Err(ProcessingError::Busy),
        }
    }
}

