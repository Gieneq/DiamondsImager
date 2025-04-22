use std::fmt::Debug;
use rand::seq::IndexedRandom;

const MULTITHREADE_ITEMS_COUNT_THRESHOLD: usize = 50;
const CONVERGE_THRESHOLD: f32 = 0.05;
const CONVERGE_ENOUGH_THRESHOLD: f32 = 0.8;
const ITERATION_MAX_COUNT: usize = 120;

/// Errors that can occur while finding centroids using the K-means algorithm.
#[derive(Debug, thiserror::Error)]
pub enum CentroidsFindError {
    /// The algorithm exceeded the maximum allowed number of iterations without converging.
    #[error("TooManyIterations")]
    TooManyIterations,

    /// The input contains fewer elements than the requested number of centroids.
    #[error("TooManyCentroids expected={expected}, actual={actual}")]
    TooManyCentroids {
        expected: usize,
        actual: usize
    },

    /// The input data is empty.
    #[error("InputEmpty")]
    InputEmpty,
}

/// Validates the input data for the K-means clustering algorithm.
///
/// The function checks two conditions:
/// - The input slice must not be empty.
/// - The number of input elements must be at least as many as `centroids_count`.
///
/// # Parameters
///
/// * `input` - A slice of input data points.
/// * `centroids_count` - The number of centroids requested.
///
/// # Returns
///
/// Returns `Ok(())` if the input is valid, or a corresponding [`CentroidsFindError`] otherwise.
fn validate_input<T>(input: &[T], centroids_count: usize) -> Result<(), CentroidsFindError> 
where 
    T: Debug + Copy + Clone
{
    if input.is_empty() {
        Err(CentroidsFindError::InputEmpty)
    } else if input.len() < centroids_count {
        Err(CentroidsFindError::TooManyCentroids { expected: centroids_count, actual: input.len() })
    } else {
        Ok(())
    }
}

/// Finds the index of the centroid closest to the given item.
///
/// # Parameters
///
/// * `item` - A reference to the data point for which the nearest centroid is required.
/// * `centroids` - A slice of candidate centroid points.
/// * `distance_measure` - A function or closure that computes the distance between two points.
///
/// # Returns
///
/// The index (in `centroids`) of the centroid that is closest to `item`.
fn find_closest_centroid_idx<T, D>(
    item: &T,
    centroids: &[T],
    distance_measure: &D
) -> usize 
where 
    T: Debug + Copy + Clone,
    D: Fn(&T, &T) -> f32,
{
    let distances_to_centroids = centroids.iter()
        .enumerate()
        .map(|(centroid_idx, centroid_value)| {
            (centroid_idx, distance_measure(item, centroid_value))
        });

    // Unwrape should be safe here, distance shoul be valid not-negative value.
    let closest_centroid = distances_to_centroids
        .min_by(|(_, a_dist), (_, b_dist)| {
            a_dist.partial_cmp(b_dist).unwrap_or_else(|| {
                panic!("Distance comparison failed at item={item:?}, a_dist={a_dist:?}, b_dist={b_dist:?}, centroids={centroids:?}");
            })
        });
    
    // Safe unwrap, assignment is not empty
    let (closest_centroid_idx, _) = closest_centroid.unwrap();
    closest_centroid_idx
}

/// Assigns each item in the input batch to the closest centroid. Used as work in the 
/// multithreaded wariant.
///
/// # Description
/// This function processes a batch of data points (`input_batch`) and assigns each point to the
/// closest centroid based on the provided `distance_measure`. The result is a vector of clusters,
/// where each cluster is a vector of data points assigned to one centroid.
///
/// # Parameters
/// * `input_batch` - A slice of data points to be assigned to clusters.
/// * `centroids` - A slice of current centroid points.
/// * `distance_measure` - A function or closure that calculates the distance between two points.
///
/// # Returns
/// A vector of clusters, where each cluster is a vector of data points assigned to one centroid.
fn get_filled_batch_cluster<T, D>(
    input_batch: &[T],
    centroids: &[T],
    distance_measure: &D
) -> Vec<Vec<T>>
where
    T: Debug + Copy + Clone + Send + Sync,
    D: Fn(&T, &T) -> f32 + Send + Sync
{
    let mut batch_clusters = vec![vec![]; centroids.len()];

    input_batch.iter().for_each(|item| {
        let closest_centroid_idx = find_closest_centroid_idx(
            item, 
            centroids, 
            distance_measure
        );
        batch_clusters[closest_centroid_idx].push(*item);
    });

    batch_clusters
}

/// Assigns items to the closest centroid using multithreading.
///
/// # Description
/// This function divides the input data into chunks, processing each chunk in parallel using multiple threads.
/// It then merges the partial results to form the final clusters. Each item in the input slice is assigned 
/// to the closest centroid based on the specified distance measure.
///
/// # Parameters
/// * `input` - A slice of data points to be assigned to clusters.
/// * `centroids` - A slice of current centroid points.
/// * `distance_measure` - A function or closure that calculates the distance between two points.
///
/// # Returns
/// A vector of clusters, where each cluster is a vector of data points assigned to one centroid.
///
/// # Multithreading Details
/// * Utilizes all available CPU cores for concurrent processing.
/// * Divides the input into `workers_count` chunks for load balancing.
/// * Aggregates the results from each thread to form the final clusters.
fn get_filled_cluster_multithreaded<T, D>(
    input: &[T],
    centroids: &[T],
    distance_measure: &D
) -> Vec<Vec<T>>
where
    T: Debug + Copy + Clone + Send + Sync,
    D: Fn(&T, &T) -> f32 + Send + Sync
{
    // Use all cores. Logical cores = doubled physical cores with hyperthreading
    let workers_count = num_cpus::get();
    let work_len = input.len();
    let work_chunk_len = work_len / workers_count;

    let ranges = (0..workers_count)
        .map(|worker_idx| {
            let from_idx = worker_idx * work_chunk_len;
            let to_idx = if worker_idx == (workers_count - 1) {
                work_len
            } else {
                from_idx + work_chunk_len
            };
            from_idx..to_idx
        })
        .collect::<Vec<_>>();

    std::thread::scope(|s| {
        let handlers = ranges.into_iter()
            .map(|range| {
                s.spawn(move || get_filled_batch_cluster(
                    &input[range.start..range.end],
                    centroids,
                    distance_measure,
                ))
            })
            .collect::<Vec<_>>();

        // Collect results
        let all_clusters = handlers.into_iter()
            .map(|handler| handler
                .join()
                .unwrap()
            )
            .collect::<Vec<_>>();
        
        // Merge results
        let mut clusters = vec![vec![]; centroids.len()];

        for partial_clusters in all_clusters {
            for (cluster_idx, partial_cluster) in partial_clusters.into_iter().enumerate() {
                clusters[cluster_idx].extend(partial_cluster);
            }
        }
        
        clusters
    })
}

/// Assigns each item in the input slice to the closest centroid.
///
/// # Description
/// This function is the entry point for cluster assignment. It assigns each data point to the closest
/// centroid by calculating distances using the provided distance measure.
///
/// It automatically selects between multithreaded and single-threaded processing based on the input size
/// and the number of available CPU cores:
/// * Uses multithreading if the input length exceeds `MULTITHREADE_ITEMS_COUNT_THRESHOLD` 
///   and there are multiple CPU cores available.
/// * Falls back to a single-threaded approach for smaller input sizes or when only one CPU core is present.
///
/// # Parameters
/// * `input` - A slice of data points to be assigned to clusters.
/// * `centroids` - A slice of current centroid points.
/// * `distance_measure` - A function or closure that calculates the distance between two points.
///
/// # Returns
/// A vector of clusters, where each cluster is a vector of data points assigned to one centroid.
///
/// # Performance
/// * Uses a multithreaded approach to leverage all CPU cores for larger input sizes.
/// * Efficiently aggregates partial results to form the final clusters.
fn create_clusters_assignment<T, D>(
    input: &[T],
    centroids: &[T],
    distance_measure: &D
) -> Vec<Vec<T>>
where
    T: Debug + Copy + Clone + Send + Sync,
    D: Fn(&T, &T) -> f32 + Send + Sync
{
    if input.len() > MULTITHREADE_ITEMS_COUNT_THRESHOLD && num_cpus::get() > 1 {
        get_filled_cluster_multithreaded(input, centroids, distance_measure)
    } else {
        get_filled_batch_cluster(input, centroids, distance_measure)
    }
}

/// Checks whether the centroids have converged.
///
/// Convergence is determined by comparing the positions of centroids between iterations.
/// If the distance between each corresponding pair of centroids is below the specified
/// `distance_threshold`, the centroids are considered converged.
///
/// # Parameters
///
/// * `last_centroids` - The centroids from the previous iteration.
/// * `recent_centroids` - The centroids from the current iteration.
/// * `distance_threshold` - The maximum allowed change for convergence.
/// * `distance_measure` - A function or closure to calculate the distance between two points.
///
/// # Returns
///
/// `true` if all centroids have converged (i.e., the distance change is below the threshold), else `false`.
fn check_converges<T, D>(
    last_centroids: &[T], 
    recent_centroids: &[T],
    distance_threshold: f32,
    distance_measure: &D
) -> bool
where 
    T: Debug + Copy + Clone,
    D: Fn(&T, &T) -> f32,
{
    let distances = last_centroids.iter()
        .zip(recent_centroids.iter())
        .map(|(last, recent)| distance_measure(last, recent))
        .collect::<Vec<_>>();

    log::info!("distances={distances:?}");

    distances.iter()
        .all(|&distance| distance < distance_threshold)
}

/// Computes new centroids by calculating the mean of each cluster.
///
/// # Parameters
///
/// * `clusters` - A slice of clusters, each represented as a vector of data points.
/// * `calculate_mean` - A function or closure that computes the mean of a slice of points.
///
/// # Returns
///
/// A vector of new centroids, each computed as the mean of the corresponding cluster.
fn create_centroids_from_clusters<T, M>(
    clusters: &[Vec<T>],
    calculate_mean: &M
) -> Vec<T>
where 
    T: Debug + Copy + Clone,
    M: Fn(&[T]) -> T
{
    clusters.iter()
        .map(|cluster| calculate_mean(cluster))
        .collect()
}

/// Performs K-means clustering to find a set of centroids for the input data.
///
/// This function implements a K-means clustering algorithm that repeatedly assigns data
/// points to the nearest centroid and then updates centroids by computing the mean of
/// the assigned points. Iteration continues until the centroids converge (i.e., change less
/// than a specified threshold) or the maximum number of iterations is reached.
///
/// # Parameters
///
/// * `input` - A slice of input data points.
/// * `centroids_count` - The number of centroids (clusters) to compute.
/// * `distance_measure` - A closure that computes the distance between two points.
/// * `calculate_mean` - A closure that computes the mean of a slice of data points.
///
/// # Returns
///
/// Returns `Ok(Vec<T>)` containing the computed centroids if the algorithm converges,
/// or a [`CentroidsFindError`] if an error occurs (e.g., too many iterations, input is empty).
///
/// # Examples
///
/// The following example demonstrates how to use `find_centroids` with floating-point data:
///
/// ```
/// use ditherum::algorithms::kmean::{find_centroids, CentroidsFindError};
///
///  // Define input data.
///  let input_data: Vec<f32> = vec![1.0, 2.0, 9.0, 7.0, 8.0, 22.0, 24.0, 3.0];
///  let centroids_count = 3;
///
///  // Define a simple absolute difference as the distance measure.
///  let distance_measure = |a: &f32, b: &f32| (a - b).abs();
///
///  // Define the mean calculation as the arithmetic mean.
///  let calculate_mean = |arr: &[f32]| arr.iter().sum::<f32>() / arr.len() as f32;
///
///  // Run the K-means clustering algorithm.
///  let centroids = find_centroids(
///      &input_data,
///      centroids_count,
///      distance_measure,
///      calculate_mean
///  );
///
///  println!("Computed centroids: {:?}", centroids);
/// ```
pub fn find_centroids<T, D, M>(
    input: &[T], 
    centroids_count: usize,
    distance_measure: D,
    calculate_mean: M

) -> Result<Vec<T>, CentroidsFindError>
where 
    T: Debug + Copy + Clone + Send + Sync,
    D: Fn(&T, &T) -> f32 + Send + Sync,
    M: Fn(&[T]) -> T
{
    validate_input(input, centroids_count)?;

    // If the number of input points equals the requested centroids count,
    // return the input data as the centroids.
    if input.len() == centroids_count {
        return Ok(input.to_vec());
    }

    let mut rng = rand::rng();

    let mut last_centroids;
    let mut centroids = input
        .choose_multiple(&mut rng, centroids_count)
        .copied()
        .collect::<Vec<_>>();
    let mut clusters;
    let mut iterations_count = 0;
    // println!("Initial centroids={centroids:?}");

    loop {
        iterations_count += 1;
        log::debug!("Iteration {iterations_count}.");

        // Assign each input point to the nearest centroid.
        clusters = create_clusters_assignment(input, &centroids, &distance_measure);
        log::trace!("Clusters: {clusters:?}");

        // Compute new centroids as the mean of the clusters.
        last_centroids = centroids;
        centroids = create_centroids_from_clusters(&clusters, &calculate_mean);

        // Check if the centroids have converged.
        if check_converges(
            &last_centroids, 
            &centroids, 
            CONVERGE_THRESHOLD,
            &distance_measure
        ) {
            log::debug!("Found solution after {iterations_count} iterations!");
            break;
        }
        
        if iterations_count > ITERATION_MAX_COUNT {
            // Iterations exhausted, but solution can be good enough
            if check_converges(
                &last_centroids, 
                &centroids, 
                CONVERGE_ENOUGH_THRESHOLD,
                &distance_measure
            ) {
                log::debug!("Found good enough solution after {iterations_count} iterations!");
                break;
            } else {
                return Err(CentroidsFindError::TooManyIterations);
            }
        }
    }

    Ok(centroids)
}         

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centroid_float() {
        let input_data: Vec<f32> = vec![1.0, 2.0, 9.0, 7.0, 8.0, 22.0, 24.0, 3.0];
        let centroids_count = 3;
        let distance_measure = |a: &f32, b: &f32| { (a - b).abs() };
        let calculate_mean = |arr: &[f32]| { arr.iter().sum::<f32>() / arr.len() as f32 };

        let centroids = find_centroids(
            &input_data, 
            centroids_count, 
            distance_measure, 
            calculate_mean
        );

        assert!(matches!(centroids, Ok(_)));
        let centroids = centroids.unwrap();
        assert_eq!(centroids.len(), centroids_count);
    }

    #[test]
    fn test_centroid_float_multithreaded() {
        let input_data: Vec<f32> = (-100..100).map(|v| v as f32).collect::<Vec<_>>();
        assert!(input_data.len() > MULTITHREADE_ITEMS_COUNT_THRESHOLD);

        let centroids_count = 5;
        let distance_measure = |a: &f32, b: &f32| { (a - b).abs() };
        let calculate_mean = |arr: &[f32]| { arr.iter().sum::<f32>() / arr.len() as f32 };

        let centroids = find_centroids(
            &input_data, 
            centroids_count, 
            distance_measure, 
            calculate_mean
        );

        assert!(matches!(centroids, Ok(_)));
        let centroids = centroids.unwrap();
        assert_eq!(centroids.len(), centroids_count);
    }
}

