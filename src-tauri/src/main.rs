// Copyright 2023-current Aris Ripandi <aris@duck.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod command;
mod decoder;
mod model;
mod player;
mod tray;
// mod utils;

use tauri::{api::shell::open, async_runtime::block_on};
use tauri::{AppHandle, Manager, SystemTrayEvent, WindowEvent};
use tauri_plugin_positioner::{Position, WindowExt};

// use anyhow::{anyhow, Context, Result};
use anyhow::{Result};
use player::Player;
use rodio::Source;
use std::{sync::Mutex};
use tokio::{time::Duration};
// use tokio::{net::TcpStream, time::sleep};
// use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
// use model::{CodeRadioMessage, Remote};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

// const WEBSOCKET_API_URL: &str = "wss://coderadio-admin.freecodecamp.org/api/live/nowplaying/coderadio";
// const REST_API_URL: &str = "https://coderadio-admin.freecodecamp.org/api/live/nowplaying/coderadio";

const LOADING_SPINNER_INTERVAL: Duration =  Duration::from_millis(60);
static PLAYER: Mutex<Option<Player>> = Mutex::new(None);
// static PROGRESS_BAR: Mutex<Option<ProgressBar>> = Mutex::new(None);

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Make the dock NOT to have an active app when started
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            // Listen for update messages
            let win = app.get_window("main").unwrap();
            win.listen("tauri://update-status".to_string(), move |msg| {
                println!("New status: {:?}", msg);
            });
            Ok(())
        })
        .plugin(tauri_plugin_positioner::init())
        .system_tray(tray::tray(VERSION))
        .on_system_tray_event(tray_event)
        .on_window_event(|event| match event.event() {
            WindowEvent::Focused(is_focused) => {
                // detect click outside of the focused window and hide the app
                if !is_focused {
                    event.window().hide().unwrap();
                }
            }
            WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![command::greet])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}

fn tray_event(app: &AppHandle, event: SystemTrayEvent) {
    tauri_plugin_positioner::on_tray_event(app, &event);
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => {
            let item_handle = app.tray_handle().get_item(&id);
            match id.as_str() {
                "play_pause" => {
                    // let selected_station: Option<Remote> = if args.select_station {
                    //     let station = select_station_interactively().await?;
                    //     Some(station)
                    // } else {
                    //     None
                    // };

                    // Connect WebSocket in background while creating `Player` to improve startup speed
                    // let websocket_connect_task = tokio::spawn(tokio_tungstenite::connect_async(WEBSOCKET_API_URL));
                    // println!("New status: {:?}", websocket_connect_task);

                    //     // let loading_spinner = ProgressBar::new_spinner()
                    //     //     .with_style(ProgressStyle::with_template("{spinner} {msg}")?)
                    //     //     .with_message("Initializing audio device...");
                    //     // loading_spinner.enable_steady_tick(LOADING_SPINNER_INTERVAL);

                    // Creating a `Player` might be time consuming. It might take several seconds on first run.
                    match Player::try_new() {
                        Ok(mut player) => {
                            // TODO replace volume value from arg
                            player.set_volume(9);
                            PLAYER.lock().unwrap().replace(player);
                        } Err(e) => { println!("ERROR: {:?}", e); }
                    }

                    // Trigger loading animation
                    block_on(set_tray_icon(app.clone())).unwrap();

                    // loading_spinner.set_message("Connecting...");
                    // let (mut websocket_stream, _) = websocket_connect_task.await??;
                    // let message = get_next_websocket_message(&mut websocket_stream).await?;
                    // loading_spinner.finish_and_clear();
                    // let stations = get_stations_from_api_message(&message);

                    // let listen_url = match selected_station {
                    //     Some(ref station) => stations
                    //         .iter()
                    //         .find(|s| s.id == station.id)
                    //         .context(anyhow!("Station with ID \"{}\" not found", station.id))?
                    //         .url
                    //         .clone(),
                    //     None => message.station.listen_url.clone(),
                    // };

                    let listen_url = "https://coderadio-admin.freecodecamp.org/radio/8010/radio.mp3";
                    println!("Playing from: {:?}", listen_url);

                    // if let Some(station) = stations.iter().find(|station| station.url == listen_url) {
                    //     writeline!("{}    {}", "Station:".bright_green(), station.name);
                    // }

                    if let Some(player) = PLAYER.lock().unwrap().as_ref() {
                        player.play(&listen_url);
                    }

                    // let mut last_song_id = String::new();
                    // update_song_info_on_screen(message, &mut last_song_id);
                    // tokio::spawn(tick_progress_bar_progress());
                    // thread::spawn(handle_keyboard_input);

                    // loop {
                    //     let message = get_next_websocket_message(&mut websocket_stream).await?;
                    //     update_song_info_on_screen(message, &mut last_song_id);
                    // }
                }
                "toggle_window" => {
                    let win = app.get_window("main").unwrap();
                    let _ = win.move_window(Position::TrayCenter);
                    let new_title = if win.is_visible().unwrap() {
                        win.hide().unwrap();
                        "Show Miniplayer"
                    } else {
                        win.show().unwrap();
                        win.set_focus().unwrap();
                        "Hide Miniplayer"
                    };
                    item_handle.set_title(new_title).unwrap();
                }
                "on_twitter" => {
                    open(&app.shell_scope(), "https://twitter.com/riipandi", None).ok();
                }
                "send_feedback" => {
                    open(
                        &app.shell_scope(),
                        "https://ripandis.com/feedback?product=soundtray",
                        None,
                    )
                    .ok();
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        }
        _ => {}
    }
}

#[tauri::command]
async fn set_tray_icon(handle: AppHandle) -> Result<(), String> {
    let mut intv = tokio::time::interval(LOADING_SPINNER_INTERVAL);
    let icon_vec = tray::tray_icon_loading();
    tokio::spawn(async move {
        let mut i = 0;
        let handle = handle.tray_handle();
        loop {
            // Wait until next tick.
            intv.tick().await;
            #[cfg(target_os = "macos")]
            handle.set_icon_as_template(false).unwrap();
            handle.set_icon(icon_vec[i].clone()).unwrap();
            i = if i >= 29 { 0 } else { i + 1 };
            // force break for test
            if i >= 29 {
                #[cfg(target_os = "macos")]
                handle.set_icon_as_template(true).unwrap();
                handle.set_icon(tray::tray_icon()).unwrap();
                break;
            }
        }
    });
    Ok(())
}
