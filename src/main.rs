#![windows_subsystem = "windows"]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(non_snake_case)]
mod notification;
mod view;
use crate::notification::notification;
use std::{fmt::Display, fs::File, io::BufReader, path::Path, process::exit, time::Duration};

use dioxus::prelude::hot_reload_init;
use dioxus_desktop::{Config, PhysicalSize, WindowBuilder};
use futures::future::join_all;
use reqwest::{ClientBuilder, Method, Request, Url};
use serde::Deserialize;
use tokio::{task, time::sleep};
use view::App;

#[derive(Debug, Default, Clone)]
pub struct UserInput {
    file_name: String,
    duration: u64,
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
    hot_reload_init!();
    dioxus_desktop::launch_cfg(
        App,
        Config::new().with_window(
            WindowBuilder::new()
                .with_title("Dead Service Notifier")
                .with_inner_size(PhysicalSize::new(600, 800)),
        ),
    );
}

async fn start(config: UserInput) {
    let tasks = load_file(&config.file_name)
        .into_iter()
        .map(|api| {
            task::spawn(async move {
                check_if_still_alive(&api, config.duration).await;
            })
        })
        .collect::<Vec<_>>();
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
        let Ok(response) = ClientBuilder::new().timeout(Duration::from_secs(3)).build().unwrap().execute(request).await else {
            notification(&api.name, &format!("主機死掉了！！！！\n{}", api.url));
            continue;
        };
        let Ok(_) = response.bytes().await else {
            notification(&api.name, &format!("服務死掉了！！！！\n{}", api.url));
            continue;
        };
    }
}
