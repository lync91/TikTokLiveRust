use env_logger::{Builder, Env}; // Importing the logger builder and environment configuration
use log::LevelFilter; // Importing log level filter
use log::{error, warn};
use std::time::Duration; // Importing Duration for timeout settings
use tiktoklive::{
    // Importing necessary modules and structs from tiktoklive crate
    core::live_client::TikTokLiveClient,
    data::live_common::{ClientData, StreamData, TikTokLiveSettings},
    errors::LibError,
    generated::events::TikTokLiveEvent,
    TikTokLive,
};
use tokio::signal; // Importing signal handling from tokio

#[tokio::main] // Main function is asynchronous and uses tokio runtime
async fn main() {
    init_logger("info"); // Initialize logger with "info" level
    let user_name = "uradatiktok";

    let client = create_client(user_name); // Create a client for the given username

    // Spawn a new asynchronous task to connect the client
    let handle = tokio::spawn(async move {
        // Attempt to connect the client
        if let Err(e) = client.connect().await {
            match e {
                // Match on the error type
                LibError::LiveStatusFieldMissing => {
                    // Specific error case
                    warn!(
                        "Failed to get live status (probably needs authenticated client): {}",
                        e
                    );
                    let auth_client = create_client_with_cookies(user_name); // Create an authenticated client
                    if let Err(e) = auth_client.connect().await {
                        // Attempt to connect the authenticated client
                        error!("Error connecting to TikTok Live after retry: {}", e);
                    }
                }
                LibError::HeaderNotReceived => {
                    error!("Error connecting to TikTok Live: {}", e);
                }

                _ => {
                    // General error case
                    error!("Error connecting to TikTok Live: {}", e);
                }
            }
        }
    });

    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C"); // Wait for Ctrl+C signal to gracefully shut down

    handle.await.expect("The spawned task has panicked"); // Await the spawned task to ensure it completes
}

fn handle_event(client: &TikTokLiveClient, event: &TikTokLiveEvent) {
    match event {
        TikTokLiveEvent::OnConnected(..) => {
            // This is an EXPERIMENTAL and UNSTABLE feature
            // Get room info from the client
            let room_info = client.get_room_info();
            // // Parse the room info
            let client_data: ClientData = serde_json::from_str(room_info).unwrap();
            // // Parse the stream data
            let stream_data: StreamData = serde_json::from_str(
                &client_data
                    .data
                    .stream_url
                    .live_core_sdk_data
                    .unwrap()
                    .pull_data
                    .stream_data,
            )
            .unwrap();
            // Get the video URL for the low definition stream with fallback to the high definition stream in a flv format
            let video_url = stream_data
                .data
                .ld
                .map(|ld| ld.main.flv)
                .or_else(|| stream_data.data.sd.map(|sd| sd.main.flv))
                .or_else(|| stream_data.data.origin.map(|origin| origin.main.flv))
                .expect("None of the stream types set");
            println!("room info: {}", video_url);
        }

        // Match on the event type
        TikTokLiveEvent::OnMember(join_event) => {
            // Handle member join event
            println!("user: {} joined", join_event.raw_data.user.nickname);
        }
        TikTokLiveEvent::OnChat(chat_event) => {
            // Handle chat event
            println!(
                "user: {} -> {}",
                chat_event.raw_data.user.nickname, chat_event.raw_data.content
            );
        }
        TikTokLiveEvent::OnGift(gift_event) => {
            // Handle gift event
            let nick = &gift_event.raw_data.user.nickname;
            let gift_name = &gift_event.raw_data.gift.name;
            let gifts_amount = gift_event.raw_data.gift.combo;
            println!(
                "user: {} sends gift: {} x {}",
                nick, gift_name, gifts_amount
            );
        }
        TikTokLiveEvent::OnLike(like_event) => {
            // Handle like event
            let nick = &like_event.raw_data.user.nickname;
            println!("user: {} likes", nick);
        }
        _ => {} // Ignore other events
    }
}

// Function to initialize the logger with a default log level
fn init_logger(default_level: &str) {
    let env = Env::default().filter_or("LOG_LEVEL", default_level); // Set default log level from environment or use provided level
    Builder::from_env(env) // Build the logger from environment settings
        .filter_module("tiktoklive", LevelFilter::Debug) // Set log level for tiktoklive module
        .init(); // Initialize the logger
}

// Function to configure the TikTok live settings
fn configure(settings: &mut TikTokLiveSettings) {
    settings.http_data.time_out = Duration::from_secs(12); // Set HTTP timeout to 12 seconds
    settings.sign_api_key = "".to_string(); // Provide your own api key here
}

// Function to configure the TikTok live settings with cookies for authentication
fn configure_with_cookies(settings: &mut TikTokLiveSettings) {
    settings.http_data.time_out = Duration::from_secs(12); // Set HTTP timeout to 12 seconds
    settings.sign_api_key = "".to_string(); // Provide your own api key here
    let contents = ""; // Placeholder for cookies
    settings
        .http_data
        .headers
        .insert("Cookie".to_string(), contents.to_string());
    // Insert cookies into HTTP headers
}

// Function to create a TikTok live client for the given username
fn create_client(user_name: &str) -> TikTokLiveClient {
    TikTokLive::new_client(user_name) // Create a new client
        .configure(configure) // Configure the client
        .on_event(handle_event) // Set the event handler
        .build() // Build the client
}

// Function to create a TikTok live client with cookies for the given username
fn create_client_with_cookies(user_name: &str) -> TikTokLiveClient {
    TikTokLive::new_client(user_name) // Create a new client
        .configure(configure_with_cookies) // Configure the client with cookies
        .on_event(handle_event) // Set the event handler
        .build() // Build the client
}
