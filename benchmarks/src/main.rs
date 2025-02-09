use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, Mutex, RwLock},
};

use rand::{rng, seq::IndexedRandom};
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
use tokio::sync::Semaphore;

// Benchmark configuration constants
const NUM_REQUESTS: usize = 1000000;
const CONCURRENCY_LIMIT: usize = 50;

#[tokio::main]
async fn main() {
    let gen = NameGenerator::new();

    let api_url = "http://localhost:8080/organization";

    let client = Client::new();
    let semaphore = Arc::new(Semaphore::new(CONCURRENCY_LIMIT));

    println!(
        "Starting benchmark: {} requests with {} concurrent workers",
        NUM_REQUESTS, CONCURRENCY_LIMIT
    );

    let durations = Arc::new(Mutex::new(Vec::new()));
    let status_counts = Arc::new(Mutex::new(HashMap::new()));

    let start_time = Instant::now();

    let mut handles = vec![];

    for _ in 0..NUM_REQUESTS {
        let client = client.clone();
        let semaphore = semaphore.clone();
        let durations = Arc::clone(&durations);
        let status_counts = Arc::clone(&status_counts);

        let curr_gen = gen.clone();
        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let request_body = format!("{{\"name\":\"{}\"}}", curr_gen.generate_name());

            let start = Instant::now();

            match client
                .post(api_url)
                .header("Content-Type", "application/json")
                .body(request_body)
                .send()
                .await
            {
                Ok(response) => {
                    let duration = start.elapsed();
                    let status_code = response.status().as_u16();
                    durations.lock().unwrap().push(duration);

                    let mut counts = status_counts.lock().unwrap();
                    *counts.entry(status_code).or_insert(0) += 1;
                }
                Err(err) => {
                    if !err.is_request() {
                        return;
                    }

                    let duration = start.elapsed();
                    let status_code = err.status().unwrap().as_u16();
                    durations.lock().unwrap().push(duration);

                    let mut counts = status_counts.lock().unwrap();
                    *counts.entry(status_code).or_insert(0) += 1;
                }
            }
        });
        handles.push(handle);
    }

    // Spawn a task to display live stats
    let durations_live = Arc::clone(&durations);
    let status_counts_live = Arc::clone(&status_counts);

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

            stdout.flush().unwrap();
        }
    });

    for handle in handles {
        let _ = handle.await;
    }

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
struct NameGenerator {
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
