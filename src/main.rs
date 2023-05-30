#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
use std::{fmt::Display, fs::File, io::BufReader, path::Path, process::exit, time::Duration};

use clap::Parser;
use futures::future::join_all;
use notify_rust::{Notification, Hint, Timeout};
use reqwest::{Method, Request, Url, Client};
use serde::Deserialize;
use tokio::{task, time::sleep};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file_path: Option<String>,

    #[arg(short, long)]
    seconds: Option<u64>,
}

#[derive(Deserialize)]
struct Api {
    name: String,
    url: String,
    method: HttpMethod,
}

#[derive(Deserialize)]
enum HttpMethod {
    Get,
    Post,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let file_path = args
        .file_path
        .unwrap_or_else(|| String::from("./api_list.json"));
    let seconds = args.seconds.map_or(5, |value| if value < 5 {5} else {value} );

    let tasks = load_file(&file_path).into_iter().map(|api| {
        task::spawn(async move {
            check_if_still_alive(&api, seconds).await;
        })
    }).collect::<Vec<_>>();
    join_all(tasks).await;
}

fn load_file<T: AsRef<Path> + Display>(file_path: &T) -> Vec<Api> {
    let Ok(buffer) = File::open(file_path).map(BufReader::new) else {
        eprintln!("Unable to open {file_path}.");
        exit(1);
    };
    let Ok(apis) = serde_json::from_reader(buffer) else {
        eprintln!("{file_path} could not be parsed.");
        exit(1);
    };
    apis
}

async fn check_if_still_alive(api: &Api, seconds: u64) {
    loop {
        sleep(Duration::from_secs(seconds)).await;
        let Ok(url) = Url::parse(&api.url) else { 
            eprintln!("Unable to parse \"{}\" as url.", api.url);
            exit(1);
        };
        let request = Request::new(
            match api.method {
                HttpMethod::Get => Method::GET,
                HttpMethod::Post => Method::POST,
            },
            url,
        );
        let Ok(response) = Client::new().execute(request).await else {
            notification(&api.name, &format!("主機死掉了！！！！（{}）", api.url));
            continue;
        };
        let Ok(_) = response.bytes().await else {
            notification(&api.name, &format!("服務死掉了！！！！（{}）", api.url));
            continue;
        };
    }
}

fn notification(name: &str, body: &str) {
    Notification::new()
        .summary(name)
        .body(body)
        .appname("Dead Service Notifier")
        .hint(Hint::Resident(true))                 
        .timeout(Timeout::Never) 
        .show()
        .unwrap();
}
