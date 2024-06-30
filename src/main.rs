/*
 * Copyright (c) 2023-2024 Marcus Butler
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

pub mod calendar;
pub mod fngbot;

use actix_web::{web, App, HttpRequest, HttpServer};
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::str;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use calendar::*;
use fngbot::*;

struct AppState {
    db_url: String,
    config: ConfigFile,
}

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    bind_address: String,
    bind_port: u16,
    debug: DebugLevel,
    f3_region: String,
    email_sender_address: String,
    email_reply_to_address: String,
    email_smtp_host: String,
    email_smtp_user: Option<String>,
    email_smtp_pass: Option<String>,
    mysql_hostname: String,
    mysql_port: u16,
    mysql_username: String,
    mysql_password: String,
    mysql_database: String,
    mailchimp_api_endpoint: String,
    mailchimp_list_id: String,
    mailchimp_api_key: String,
    slack_api_key: String,
    slack_channel_id: String,
    slack_signing_secret: String,
    slack_invite_link: String,
}

#[derive(Deserialize,Debug)]
pub enum DebugLevel {
    Info,
    Debug,
    Trace,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut handle = File::open("config.toml").expect("Error opening config file");
    let mut config_text = String::new();
    let _ = handle.read_to_string(&mut config_text);
    let config: ConfigFile = toml::from_str(&config_text).expect("Error parsing config file");

    let tracing_level = match config.debug {
        DebugLevel::Info => Level::INFO,
        DebugLevel::Debug => Level::DEBUG,
        DebugLevel::Trace => Level::TRACE,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing_level)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Could not set default global tracing subscriber");

    info!("Starting tracing log for F3 Utils");

    let bind_address = config.bind_address.clone();
    let bind_port = config.bind_port;

    let db_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        config.mysql_username,
        config.mysql_password,
        config.mysql_hostname,
        config.mysql_port,
        config.mysql_database
    );

    let state = web::Data::new(AppState { db_url, config });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(calendar_service)
            .service(fngbot_service)
    })
    .bind((bind_address, bind_port))?
    .run()
    .await
}

fn get_client_ip(req: &HttpRequest) -> String {
    if let Some(ip) = req.headers().get("x-forwarded-for") {
        if let Ok(ip_str) = ip.to_str() {
            String::from(ip_str)
        } else {
            String::from("")
        }
    } else if let Some(ip) = req.headers().get("x-real-ip") {
        if let Ok(ip_str) = ip.to_str() {
            String::from(ip_str)
        } else {
            String::from("")
        }
    } else if let Some(ip) = req.peer_addr() {
        ip.ip().to_string()
    } else {
        String::from("")
    }
}
