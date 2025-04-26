use std::{
    fmt::Debug, 
    sync::Arc
};

use crate::services::{
    dmc::{
        DmcBom, 
        PaletteDmc
    }, 
    processing::image_manip::image_dither_using_dmc_palette
};

const WORKER_QUEUE_CAP: usize = 4;

/// Identifier for a piece of work. A dispatcher of work 
/// should make them unique to easy distinguish of workers.
pub type WorkId = u64;

/// A unit of work that can be processed by a `Worker`.
pub enum Work {
    /// Extract a color palette from an image.
    ///
    /// - `palette_dmc`: reference DMC palette
    /// - `src_image`: source image to sample
    /// - `max_colors`: optional cap on number of colors to extract
    PaletteExtract {
        palette_dmc: Arc<PaletteDmc>, 
        src_image: Arc<image::RgbImage>,
        max_colors: Option<usize>
    },

    /// Apply Floyd–Steinberg dithering using the given DMC palette.
    ///
    /// - `palette_dmc`: reference DMC palette
    /// - `src_image`: source image to dither
    ImageDither {
        palette_dmc: Arc<PaletteDmc>, 
        src_image: Arc<image::RgbImage>,
    },

    /// A dummy test workload that sleeps for a duration.
    /// Only available under `cfg(test)`.
    #[cfg(test)]
    TestWork {
        delay: std::time::Duration
    }
}

impl Debug for Work {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Work::PaletteExtract { palette_dmc, src_image, max_colors } => {
                f.debug_struct("PaletteExtract")
                    .field("palette_dmc", &format_args!("Palette of {} DMCs", palette_dmc.len()))
                    .field("image_size", &format_args!("{}x{}", src_image.width(), src_image.height()))
                    .field("max_colors", max_colors)
                    .finish()
            },
            Work::ImageDither { palette_dmc, src_image } => {
                f.debug_struct("ImageDither")
                    .field("palette_dmc", &format_args!("Palette of {} DMCs", palette_dmc.len()))
                    .field("image_size", &format_args!("{}x{}", src_image.width(), src_image.height()))
                    .finish()
            },
            #[cfg(test)]
            Work::TestWork { delay } => {
                f.debug_struct("TestWork")
                    .field("delay", delay)
                    .finish()
            },
        }
    }
}

/// A `Work` message tagged with its `WorkId`.
#[derive(Debug)]
pub struct WorkWrapped {
    pub id: WorkId,
    pub work: Work,
}

/// The outcome of a completed `Work`.
pub enum WorkResult {
    PaletteExtract {
        dmc_bom: DmcBom,
    },
    ImageDither {
        dithered_image: image::RgbImage,
        dmc_bom: DmcBom,
    },
    #[cfg(test)]
    TestWork,
}

impl Debug for WorkResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkResult::PaletteExtract { dmc_bom } => {
                f.debug_struct("PaletteExtract")
                    .field("dmc_bom", &format_args!("BOM of {} DMCs", dmc_bom.len()))
                    .finish()
            },
            WorkResult::ImageDither { dithered_image, dmc_bom } => {
                f.debug_struct("ImageDither")
                    .field("dithered_image_size", &format_args!("{}x{}", dithered_image.width(), dithered_image.height()))
                    .field("dmc_bom", &format_args!("BOM of {} DMCs", dmc_bom.len()))
                    .finish()
            },
            #[cfg(test)]
            WorkResult::TestWork => {
                f.debug_struct("TestWork")
                    .finish()
            },
        }
    }
}

/// A `WorkResult` tagged with its originating `WorkId`.
#[derive(Debug)]
pub struct WorkResultWrapped {
    pub id: WorkId,
    pub work_result: WorkResult,
}

/// A background worker that processes `WorkWrapped` messages on a blocking thread
/// and sends back `WorkResultWrapped` messages.
#[derive(Debug)]
pub struct Worker {
    pub task: tokio::task::JoinHandle<()>,
    pub work_tx: tokio::sync::mpsc::Sender<WorkWrapped>,
    pub work_result_rx: tokio::sync::mpsc::Receiver<WorkResultWrapped>,
}

impl Worker {
    /// Execute a single `WorkWrapped`, spawning blocking work and returning the result.
    async fn do_work(work_to_do: WorkWrapped) -> WorkResult {
        let span = tracing::info_span!("worker.do_work", id = work_to_do.id);
        let _span_enter = span.enter();
        
        let work = work_to_do.work;
        tracing::info!("Start doing processing {work:?}...");

        // Expect this task will not fail or abort. It should finish its work in finite time.
        let result = tokio::task::spawn_blocking(move || {
             match work {
                Work::PaletteExtract { palette_dmc, src_image, max_colors } => {
                    let dmc_counts = palette_dmc.find_subset_closest_to_image_pixels(&src_image, max_colors);
                    WorkResult::PaletteExtract { dmc_bom: dmc_counts }
                },
                Work::ImageDither { palette_dmc, src_image } => {
                    let (dithered_image, dmc_bom) = image_dither_using_dmc_palette(&palette_dmc, &src_image);
                    WorkResult::ImageDither { dithered_image, dmc_bom }
                },
                #[cfg(test)]
                Work::TestWork { delay } => {
                    std::thread::sleep(delay);
                    WorkResult::TestWork
                },
            }
        }).await
        .expect("blocking task panicked");

        tracing::info!("Finished doing processing, result = {result:?}!");
        result
    }
    
    /// Create a new worker with the given `id`.
    ///
    /// Spawns a background task that pulls work from the queue,
    /// executes it via `do_work`, and forwards results to the result channel.
    pub fn new(id: u32) -> Self {
        let (work_tx, mut work_rx) = tokio::sync::mpsc::channel::<WorkWrapped>(WORKER_QUEUE_CAP);
        let (work_result_tx, work_result_rx) = tokio::sync::mpsc::channel(WORKER_QUEUE_CAP);

        let task = tokio::task::spawn(async move {
            // Shutting down signal using `work_rx`
            loop {
                match work_rx.recv().await {
                    Some(work_to_do) => {
                        let work_id = work_to_do.id;
                        let work_result = Self::do_work(work_to_do).await;

                        let work_result_wrapped = WorkResultWrapped {
                            id: work_id,
                            work_result
                        };

                        if let Err(_) = work_result_tx.send(work_result_wrapped).await {
                            tracing::info!("Stopping worker {id}, noone need results of his work :(");
                            break;
                        }
                    },
                    None => {
                        tracing::info!("Stopping worker {id}");
                        break;
                    }
                }
            }
        });

        Self { 
            task, 
            work_tx,
            work_result_rx
        }
    }

    /// Attempt to enqueue `work_wrapped` without waiting.
    /// Returns an error if the queue is full.
    pub fn try_enque_work(&self, work_wrapped: WorkWrapped) -> Result<(), tokio::sync::mpsc::error::TrySendError<WorkWrapped>> {
        self.work_tx.try_send(work_wrapped)
    }

    /// Receive the next completed `WorkResultWrapped`.
    /// Returns `None` if the worker has shut down and all results are drained.
    pub async fn recv_work_result(&mut self) -> Option<WorkResultWrapped> {
        self.work_result_rx.recv().await
    }
}

// Run with: cargo test tests_worker -- --nocapture
#[cfg(test)]
mod tests_worker {
    use super::*;
    use std::time::Duration;
    use tracing_subscriber;

    fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_test_writer() // Output for tests
            .try_init();
    }

    #[tokio::test]
    async fn test_worker_dummy_work() {
        {
            init_tracing();
            
            let mut worker = Worker::new(0);
            let enque_result = worker.try_enque_work(WorkWrapped { id: 12, work: Work::TestWork { delay: Duration::from_millis(500) } });
            assert!(enque_result.is_ok());

            let work_result = worker.recv_work_result().await.unwrap();
            assert!(matches!(work_result, WorkResultWrapped { id: 12, work_result: WorkResult::TestWork }));
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    #[tokio::test]
    async fn test_worker_palette_extraction() {
        {
            init_tracing();

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
            // Etracted colors are hard to predict
            assert!(matches!(work_result, WorkResultWrapped { id: 13, work_result: WorkResult::PaletteExtract { dmc_bom: _ } }));
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    #[tokio::test]
    async fn test_worker_image_dithering() {
        {
            init_tracing();

            let palette_dmc = Arc::new(PaletteDmc::load_from_file_default().unwrap());
            let src_image = Arc::new(ditherum::image_utils::generate_gradient_image(
                400,
                4,
                image::Rgb([0,33,255]),
                image::Rgb([255,55,0]),
            ));

            let mut worker = Worker::new(2);

            let work = WorkWrapped {
                id: 14,
                work: Work::ImageDither { palette_dmc: palette_dmc.clone(), src_image }
            };

            let enque_result = worker.try_enque_work(work);
            assert!(enque_result.is_ok());

            let work_result = worker.recv_work_result().await.unwrap();
            if let WorkResult::ImageDither { dithered_image, dmc_bom } = work_result.work_result {
                let pixels_count = dithered_image.width() * dithered_image.height();
                let diamonds_used_count = dmc_bom.iter().fold(0, |acc, (_, cnt)| acc + cnt);
                assert_eq!(pixels_count, diamonds_used_count);
                
                let total_colors_count = palette_dmc.len();
                let used_colors_count = dmc_bom.len();
                
                // Used colors are hard to predict
                assert!(used_colors_count < total_colors_count);
                println!("Used colors: {used_colors_count}/{total_colors_count}")

            } else {
                panic!("Bad work result = {work_result:?}")
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}