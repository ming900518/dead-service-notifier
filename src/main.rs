#![windows_subsystem = "windows"]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
use std::{fmt::Display, fs::File, io::BufReader, path::Path, process::exit, time::Duration};

use clap::Parser;
use futures::future::join_all;
use reqwest::{ClientBuilder, Method, Request, Url};
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

#[cfg(target_os = "linux")]
#[tokio::main]
async fn main() {
    use rfd::AsyncFileDialog;
    let args = Args::parse();
    let file_path = if let Some(file_path) = args.file_path {
        file_path
    } else {
        AsyncFileDialog::new()
            .add_filter("JSON", &["json"])
            .set_directory("/")
            .pick_file()
            .await
            .unwrap()
            .path()
            .to_str()
            .unwrap()
            .to_string()
    };
    let seconds = args
        .seconds
        .map_or(5, |value| if value < 5 { 5 } else { value });

    let tasks = load_file(&file_path)
        .into_iter()
        .map(|api| {
            task::spawn(async move {
                check_if_still_alive(&api, seconds).await;
            })
        })
        .collect::<Vec<_>>();
    join_all(tasks).await;
}

#[cfg(target_os = "macos")]
fn main() {
    use cacao::{
        appkit::{
            menu::{Menu, MenuItem},
            window::{Window, WindowConfig, WindowDelegate},
            App, AppDelegate,
        },
        filesystem::FileSelectPanel,
        notification_center::Dispatcher,
    };
    struct ThisApp {
        window: Window<FileSelect>,
    }

    struct FileSelect {
        panel: FileSelectPanel,
    }

    impl AppDelegate for ThisApp {
        fn did_finish_launching(&self) {}
        fn should_terminate_after_last_window_closed(&self) -> bool {
            false
        }
    }

    enum Message {
        Selected,
    }

    impl Dispatcher for ThisApp {
        type Message = Message;
        fn on_ui_message(&self, message: Self::Message) {
            if matches!(message, Message::Selected) {
                self.window.close();
            }
        }
    }

    impl WindowDelegate for FileSelect {
        const NAME: &'static str = "jsonSelect";
        fn did_load(&mut self, window: Window) {
            window.show();
            self.panel.set_can_choose_files(true);
            self.panel.set_can_choose_directories(false);
            self.panel.set_resolves_aliases(true);
            self.panel.set_allows_multiple_selection(false);
            self.panel.begin_sheet(&window, |event| {
                if event.first().is_some() {
                    let pathbuf = event.first().unwrap().pathbuf();
                    std::thread::spawn(move || {
                        let new_path = pathbuf.to_str().unwrap();
                        tokio::runtime::Builder::new_multi_thread()
                            .enable_all()
                            .build()
                            .unwrap()
                            .block_on(async {
                                let tasks = load_file(&new_path)
                                    .into_iter()
                                    .map(|api| {
                                        task::spawn(async move {
                                            check_if_still_alive(&api, 5).await;
                                        })
                                    })
                                    .collect::<Vec<_>>();
                                join_all(tasks).await;
                            });
                    });
                    App::<ThisApp, Message>::dispatch_main(Message::Selected);
                }
            });
        }
    }
    App::set_menu(vec![Menu::new("", vec![MenuItem::Quit])]);
    App::activate();
    App::new(
        "tw.mingchang.deadservicenotifier",
        ThisApp {
            window: Window::with(
                WindowConfig::default(),
                FileSelect {
                    panel: FileSelectPanel::default(),
                },
            ),
        },
    )
    .run();
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

#[cfg(target_os = "linux")]
fn notification(name: &str, body: &str) {
    use notify_rust::{Hint, Notification, Timeout};

    Notification::new()
        .summary(name)
        .body(body)
        .appname("Dead Service Notifier")
        .hint(Hint::Resident(true))
        .timeout(Timeout::Never)
        .show()
        .unwrap();
}

#[cfg(target_os = "macos")]
fn notification(name: &str, body: &str) {
    use cacao::user_notifications::{Notification, NotificationAuthOption, NotificationCenter};
    NotificationCenter::request_authorization(&[
        NotificationAuthOption::Alert,
        NotificationAuthOption::Sound,
        NotificationAuthOption::Badge,
    ]);
    NotificationCenter::notify(Notification::new(name, body));
}

#[cfg(target_os = "windows")]
fn notification(name: &str, body: &str) {
    use winrt_notification::{Duration, Sound, Toast};

    Toast::new(Toast::POWERSHELL_APP_ID)
        .title("Dead Service Notifier")
        .text1(name)
        .text2(body)
        .sound(Some(Sound::SMS))
        .duration(Duration::Short)
        .show()
        .unwrap();
}
