use std::{collections::HashMap, sync::Arc};

use crate::services::dmc::{Dmc, PaletteDmc};

const WORKER_QUEUE_CAP: usize = 4;

#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("Work ould not be finished reson='{0}'")]
    WorkCouldNotBeFinished(#[from] tokio::task::JoinError),
}

pub type WorkId = u32;

#[derive(Debug)]
pub enum Work {
    PaletteExtract {
        palette_dmc: Arc<PaletteDmc>, 
        src_image: Arc<image::RgbImage>,
        max_colors: Option<usize>
    },
    TestWork {
        delay: std::time::Duration
    }
}

#[derive(Debug)]
pub struct WorkWrapped {
    pub id: WorkId,
    pub work: Work,
}

#[derive(Debug)]
pub enum WorkResult {
    PaletteExtract {
        dmc_counts: HashMap<Dmc, u32>,
    },
    TestWork,
}

#[derive(Debug)]
pub struct WorkResultWrapped {
    pub id: WorkId,
    pub work_result: Result<WorkResult, WorkerError>,
}

#[derive(Debug)]
pub struct Worker {
    id: u32,
    pub task: tokio::task::JoinHandle<()>,
    pub work_tx: tokio::sync::mpsc::Sender<WorkWrapped>,
    pub work_result_rx: tokio::sync::mpsc::Receiver<WorkResultWrapped>,
}

impl Worker {
    async fn do_work(work_to_do: WorkWrapped) -> Result<WorkResult, WorkerError> {
        tracing::info!("Start doing processing {work_to_do:?}");
        println!("Start doing processing {work_to_do:?}");
        let work = work_to_do.work;

        let result = tokio::task::spawn_blocking(move || {
             match work {
                Work::PaletteExtract { palette_dmc, src_image, max_colors } => {
                    let dmc_counts = palette_dmc.find_subset_matching_image(&src_image, max_colors);
                    WorkResult::PaletteExtract { dmc_counts }
                },
                Work::TestWork { delay } => {
                    std::thread::sleep(delay);
                    WorkResult::TestWork
                },
            }
        }).await;

        result.map_err(WorkerError::from)
    }
    
    pub fn new(id: u32) -> Self {
        let (work_tx, mut work_rx) = tokio::sync::mpsc::channel::<WorkWrapped>(WORKER_QUEUE_CAP);
        let (work_result_tx, work_result_rx) = tokio::sync::mpsc::channel(WORKER_QUEUE_CAP);


        let task = tokio::task::spawn(async move {
            
            loop {
                tokio::select! {
                    work = work_rx.recv() => match work {
                        Some(work_to_do) => {
                            let work_id = work_to_do.id;
                            let work_result = Self::do_work(work_to_do).await;

                            let work_result_wrapped = WorkResultWrapped {
                                id: work_id,
                                work_result
                            };

                            if let Err(_) = work_result_tx.send(work_result_wrapped).await {
                                println!("Stopping worker {id}, noone need results of his work :(");
                                break;
                            }
                        },
                        None => {
                            tracing::info!("Stopping worker {id}");
                            println!("Stopping worker {id}");
                            break;
                        }
                    }
                }
            }

        });

        Self { 
            id,
            task, 
            work_tx,
            work_result_rx
        }
    }

    pub fn try_enque_work(&self, work_wrapped: WorkWrapped) -> Result<(), tokio::sync::mpsc::error::TrySendError<WorkWrapped>> {
        self.work_tx.try_send(work_wrapped)
    }

    pub async fn recv_work_result(&mut self) -> Option<WorkResultWrapped> {
        self.work_result_rx.recv().await
    }

}

#[cfg(test)]
mod tests_worker {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_worker_dummy_work() {
        {
            let mut worker = Worker::new(0);
            let enque_result = worker.try_enque_work(WorkWrapped { id: 12, work: Work::TestWork { delay: Duration::from_millis(500) } });
            assert!(enque_result.is_ok());

            let work_result = worker.recv_work_result().await.unwrap();
            assert!(matches!(work_result, WorkResultWrapped { id: 12, work_result: Ok(WorkResult::TestWork)}));
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    #[tokio::test]
    async fn test_worker_palette_extraction() {
        {
            let max_colors = Some(10);
            let palette_dmc = Arc::new(PaletteDmc::load_from_file_default().unwrap());
            let src_image = Arc::new(ditherum::image_utils::generate_gradient_image(
                100,
                20,
                image::Rgb([0,0,0]),
                image::Rgb([255,0,0]),
            ));

            let mut worker = Worker::new(1);

            let work = WorkWrapped {
                id: 13,
                work: Work::PaletteExtract { palette_dmc, src_image, max_colors }
            };

            let enque_result = worker.try_enque_work(work);
            assert!(enque_result.is_ok());

            let work_result = worker.recv_work_result().await.unwrap();
            assert!(matches!(work_result, WorkResultWrapped { id: 13, work_result: Ok(WorkResult::PaletteExtract { dmc_counts: _ })}));
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}