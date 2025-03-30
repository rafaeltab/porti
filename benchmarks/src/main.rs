use std::{
    fs::File,
    io::BufReader,
    sync::{atomic::AtomicU64, Arc, LazyLock, Mutex, RwLock},
};

use rand::{rng, seq::IndexedRandom};
use requests::{
    delete_platform_account::DeletePlatformAccount, get_organization::GetOrganization,
    get_organization_log::GetOrganizationLog, get_organizations::GetOrganizations,
    openapi::OpenApi, post_organizations::PostOrganizations,
    post_platform_accounts::PostPlatformAccounts, Request, RequestHandler,
};
use serde::Deserialize;

use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};
use reqwest::Client;
use std::collections::HashMap;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};
use store::Store;
use tokio::sync::Semaphore;

pub mod requests;
pub mod store;

const LOGGING: bool = true;

// Benchmark configuration constants
const NUM_REQUESTS: usize = 100000000;
const CONCURRENCY_LIMIT: usize = 300;

static SEMAPHORE: LazyLock<Arc<Semaphore>> = LazyLock::new(|| Arc::new(Semaphore::new(CONCURRENCY_LIMIT)));

struct ImplRequestHandler {
    durations: Arc<Mutex<Vec<Duration>>>,
    status_counts: Arc<Mutex<HashMap<u16, i32>>>,
    start: RwLock<Option<Instant>>,
}

impl RequestHandler for ImplRequestHandler {
    fn handle_request(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        {
            let mut a = self.start.write().unwrap();
            *a = Some(Instant::now());
        }
        request
    }

    fn handle_response(
        &self,
        response: Result<reqwest::Response, reqwest::Error>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let duration = self.start.read().unwrap().unwrap().elapsed();

        match &response {
            Ok(response) => {
                let status_code = response.status().as_u16();
                self.durations.lock().unwrap().push(duration);

                let mut counts = self.status_counts.lock().unwrap();
                *counts.entry(status_code).or_insert(0) += 1;
            }
            Err(err) => {
                if !err.is_request() {
                    return response;
                }

                let status_code = err.status().unwrap().as_u16();
                self.durations.lock().unwrap().push(duration);

                let mut counts = self.status_counts.lock().unwrap();
                *counts.entry(status_code).or_insert(0) += 1;
            }
        };

        response
    }
}

#[tokio::main]
async fn main() {
    let gen = Arc::new(NameGenerator::new());
    let store = Arc::new(Store::default());
    store.run().await;
    let root_url = Arc::new("http://localhost:8080".to_string());

    let requests: Arc<Vec<Box<dyn Request + Send + Sync>>> = Arc::new(vec![
        Box::new(DeletePlatformAccount {
            store: store.clone(),
            root_url: root_url.clone(),
        }),
        Box::new(GetOrganization {
            store: store.clone(),
            root_url: root_url.clone(),
        }),
        Box::new(GetOrganizationLog {
            store: store.clone(),
            root_url: root_url.clone(),
        }),
        Box::new(GetOrganizations {
            store: store.clone(),
            root_url: root_url.clone(),
        }),
        Box::new(OpenApi {
            store: store.clone(),
            root_url: root_url.clone(),
        }),
        Box::new(PostOrganizations {
            store: store.clone(),
            root_url: root_url.clone(),
            name_generator: gen.clone(),
        }),
        Box::new(PostPlatformAccounts {
            store: store.clone(),
            root_url: root_url.clone(),
            name_generator: gen.clone(),
        }),
    ]);

    let client = Client::new();

    if LOGGING {
        println!(
            "Starting benchmark: {} requests with {} concurrent workers",
            NUM_REQUESTS, CONCURRENCY_LIMIT
        );
    }

    let durations = Arc::new(Mutex::new(Vec::new()));
    let status_counts = Arc::new(Mutex::new(HashMap::new()));

    let start_time = Instant::now();

    let mut count_map = HashMap::new();
    let mut total_count_map = HashMap::new();
    for i in 0..requests.len() {
        count_map.insert(i, AtomicU64::new(0));
        total_count_map.insert(i, AtomicU64::new(0));
    }

    let count = Arc::new(count_map);
    let total_counts = Arc::new(total_count_map);


    // Spawn a task to display live stats
    let durations_live = Arc::clone(&durations);
    let status_counts_live = Arc::clone(&status_counts);

    if LOGGING {
        let count = count.clone();
        let total_counts = total_counts.clone();
        tokio::spawn(async move {
            let mut stdout = stdout();
            loop {
                tokio::time::sleep(Duration::from_millis(500)).await;

                let durations = durations_live.lock().unwrap();
                let status_counts = status_counts_live.lock().unwrap();

                execute!(stdout, Clear(ClearType::All), MoveTo(0, 0)).unwrap();
                writeln!(stdout, "Live Benchmark Stats:").unwrap();

                if !durations.is_empty() {
                    let avg_duration: Duration =
                        durations.iter().sum::<Duration>() / (durations.len() as u32);
                    writeln!(stdout, "Average request duration: {:.2?}", avg_duration).unwrap();

                    let percentiles = [0.90, 0.99, 0.999];
                    for &p in &percentiles {
                        let index = (p * durations.len() as f64).ceil() as usize - 1;
                        let index = index.min(durations.len() - 1);
                        writeln!(
                            stdout,
                            "{}% percentile: {:.2?}",
                            p * 100.0,
                            durations[index]
                        )
                        .unwrap();
                    }

                    let elapsed = start_time.elapsed().as_secs_f64();
                    let requests_per_second = durations.len() as f64 / elapsed;
                    writeln!(stdout, "Requests per second: {:.2}", requests_per_second).unwrap();
                }

                // Display status codes received
                writeln!(stdout, "\nResponse Codes:").unwrap();
                for (status_code, count) in status_counts.iter() {
                    writeln!(stdout, "{}: {} requests", status_code, count).unwrap();
                }

                // Display active requests
                writeln!(stdout, "\nActive requests:").unwrap();
                for (index, count) in count.iter() {
                    writeln!(
                        stdout,
                        "{}: {} requests",
                        index,
                        count.load(std::sync::atomic::Ordering::Relaxed)
                    )
                    .unwrap();
                }

                writeln!(stdout, "\n Total requests:").unwrap();
                for (index, count) in total_counts.iter() {
                    writeln!(
                        stdout,
                        "{}: {} requests",
                        index,
                        count.load(std::sync::atomic::Ordering::Relaxed)
                    )
                    .unwrap();
                }

                stdout.flush().unwrap();
            }
        });
    }

    for _ in 0..NUM_REQUESTS {
        let requests = requests.clone();

        let client = client.clone();
        let durations: Arc<Mutex<Vec<Duration>>> = Arc::clone(&durations);
        let status_counts: Arc<Mutex<HashMap<u16, i32>>> = Arc::clone(&status_counts);
        let count = count.clone();
        let total_counts = total_counts.clone();

        let _permit = SEMAPHORE.acquire().await.unwrap();
        tokio::spawn(async move {
            let mut lowest_val = u64::MAX;
            let mut lowest_index: usize = 0;

            for (i, c) in count.iter() {
                let val = c.load(std::sync::atomic::Ordering::Relaxed);
                if val < lowest_val {
                    lowest_val = val;
                    lowest_index = *i;
                }
            }

            let req_index = lowest_index;

            let req = requests.get(req_index).unwrap();

            let counter = count.get(&req_index).unwrap();
            let total_counter = total_counts.get(&req_index).unwrap();

            let handler = ImplRequestHandler {
                start: RwLock::new(None),
                status_counts: status_counts.clone(),
                durations: durations.clone(),
            };

            counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            total_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            req.make_request(client, Box::new(handler)).await;

            counter.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);

            std::mem::drop(_permit);
        });
    }

    let _ = SEMAPHORE.acquire_many(CONCURRENCY_LIMIT as u32).await;

    let total_duration = start_time.elapsed();

    // Final stats
    println!("\nBenchmark completed in {:.2?}", total_duration);

    let mut durations = durations.lock().unwrap();
    durations.sort();

    if !durations.is_empty() {
        let avg_duration: Duration = durations.iter().sum::<Duration>() / (durations.len() as u32);
        println!("Average request duration: {:.2?}", avg_duration);

        let percentiles = [0.90, 0.99, 0.999];
        for &p in &percentiles {
            let index = (p * durations.len() as f64).ceil() as usize - 1;
            let index = index.min(durations.len() - 1); // Ensure valid index
            println!("{}% percentile: {:.2?}", p * 100.0, durations[index]);
        }

        println!(
            "Requests per second: {:.2}",
            NUM_REQUESTS as f64 / total_duration.as_secs_f64()
        );
    } else {
        println!("No successful requests recorded.");
    }

    // Print status code overview
    println!("\nResponse Codes:");
    for (status_code, count) in status_counts.lock().unwrap().iter() {
        println!("{}: {} requests", status_code, count);
    }
}

#[derive(Debug, Deserialize)]
struct NameLists {
    first_names: Vec<String>,
    middle_names: Vec<String>,
    last_names: Vec<String>,
}

#[derive(Clone)]
pub struct NameGenerator {
    data: Arc<RwLock<NameLists>>,
}

impl NameGenerator {
    // Load the JSON files at the start
    pub fn new() -> Self {
        let first_names = Self::load_json("files/first-names.json");
        let middle_names = Self::load_json("files/middle-names.json");
        let last_names = Self::load_json("files/names.json");

        let data = NameLists {
            first_names,
            middle_names,
            last_names,
        };

        NameGenerator {
            data: Arc::new(RwLock::new(data)),
        }
    }

    // Helper to load a JSON file into a vector
    pub fn load_json(file_path: &str) -> Vec<String> {
        let file = File::open(file_path).expect("Unable to open file");
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).expect("Error reading JSON data")
    }

    // Efficiently generate a random name
    pub fn generate_name(&self) -> String {
        let data = self.data.read().unwrap();
        let first_name = data.first_names.choose(&mut rng()).unwrap();
        let middle_name = data.middle_names.choose(&mut rng()).unwrap();
        let last_name = data.last_names.choose(&mut rng()).unwrap();

        format!("{first_name} {middle_name} {last_name}")
    }
}

impl Default for NameGenerator {
    fn default() -> Self {
        Self::new()
    }
}
