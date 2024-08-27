use std::env;
use clap::{App, Arg};
use reqwest::*;
use serde::Deserialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::ops::Add;
use std::path::Path;
use chrono::{TimeDelta, Utc};
use parse_link_header::parse_with_rel;

#[derive(Debug, Deserialize)]
struct DemodData {
    payload_demod: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Observation {
    id: i32,
    start: Option<String>,
    end: Option<String>,
    ground_station: Option<i32>,
    transmitter: Option<String>,
    norad_cat_id: Option<i32>,
    payload: Option<String>,
    waterfall: Option<String>,
    demoddata: Vec<DemodData>,
    station_name: Option<String>,
    station_lat: Option<f64>,
    station_lon: Option<f64>,
    station_alt: Option<i32>,
    vetted_status: Option<String>,
    vetted_user: Option<i32>,
    vetted_datetime: Option<String>,
    archived: Option<bool>,
    archive_url: Option<String>,
    client_version: Option<String>,
    client_metadata: Option<String>,
    status: Option<String>,
    waterfall_status: Option<String>,
    waterfall_status_user: Option<i32>,
    waterfall_status_datetime: Option<String>,
    rise_azimuth: Option<f64>,
    set_azimuth: Option<f64>,
    max_altitude: Option<f64>,
    transmitter_uuid: Option<String>,
    transmitter_description: Option<String>,
    transmitter_type: Option<String>,
    transmitter_uplink_low: Option<i32>,
    transmitter_uplink_high: Option<i32>,
    transmitter_uplink_drift: Option<i32>,
    transmitter_downlink_low: Option<i32>,
    transmitter_downlink_high: Option<i32>,
    transmitter_downlink_drift: Option<i32>,
    transmitter_mode: Option<String>,
    transmitter_invert: Option<bool>,
    transmitter_baud: Option<f64>,
    transmitter_updated: Option<String>,
    transmitter_status: Option<String>,
    tle0: Option<String>,
    tle1: Option<String>,
    tle2: Option<String>,
    center_frequency: Option<i32>,
    observer: Option<String>,
    observation_frequency: Option<i32>,
    transmitter_unconfirmed: Option<bool>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("satnogs_downloader")
        .version("0.1.0")
        .author("Patrick Oppel")
        .about("Downloads satnogs observations")
        .arg(Arg::with_name("start_date")
            .short('s')
            .long("start_date")
            .value_name("START_DATE")
            .help("Start date")
            .takes_value(true))
        .arg(Arg::with_name("end_date")
            .short('e')
            .long("end_date")
            .value_name("END_DATE")
            .help("End date")
            .takes_value(true))
        .arg(Arg::with_name("satellite_id")
            .short('i')
            .long("satellite_id")
            .value_name("SATELLITE_ID")
            .help("Satellite ID")
            .takes_value(true))
        .get_matches();

    let start_date = matches.value_of("start_date").unwrap_or("2024-08-16");
    let default_end_date = format!("{}", Utc::now().add(TimeDelta::days(1)).format("%F"));
    let end_date = matches.value_of("end_date").unwrap_or(&default_end_date);
    let satellite_id = matches.value_of("satellite_id").unwrap_or("98858");
    let api_token = env::var("SATNOGS_API_TOKEN").expect("Provide SATNOGS_API_TOKEN env var");

    let mut url = format!("https://network.satnogs.org/api/observations/?start={}&end={}&satellite__norad_cat_id={}&status=good&format=json", start_date, end_date, satellite_id);

    let client = Client::new();

    loop {
        println!("Querying API: {}", url);
        let response = client.get(&url)
            .header(header::AUTHORIZATION, format!("Bearer: {}", api_token))
            .send()
            .await?;

        let links = response.headers().get("Link");
        let mut has_next = false;
        if let Some(links) = links {
            let links_str = links.to_str().unwrap();
            println!("Found links: {}", links_str);
            if let Ok(links) = parse_with_rel(links_str) {
                if let Some(link) = links.get("next") {
                    url = link.raw_uri.to_string();
                    has_next = true;
                }
            }
        }

        let observations: Vec<Observation> = serde_json::from_str(&response.text().await?).unwrap();

        for observation in observations {
            if observation.demoddata.is_empty() {
                println!("Skipping observation with no data: {}", observation.id);
            } else {
                println!("Downloading observation: {}", observation.id);
                let mut url = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(Path::new(&format!("./{}.url", observation.id))).expect("Unable to create file");

                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(Path::new(&format!("./{}.raw", observation.id))).expect("Unable to create file");

                for demod_data in observation.demoddata {
                    let response = client.get(&demod_data.payload_demod).send().await?;
                    file.write_all(response.bytes().await?.as_ref()).expect("Unable to write data");
                    url.write_all(format!("{:?}\n", demod_data.payload_demod).as_bytes()).expect("Unable to write data");
                }

                // move files to folder
                std::fs::create_dir_all(format!("./{}", satellite_id)).expect("Unable to create folder");
                std::fs::rename(format!("./{}.url", observation.id), format!("./{}/{}.url", satellite_id, observation.id)).expect("Unable to move file");
                std::fs::rename(format!("./{}.raw", observation.id), format!("./{}/{}.raw", satellite_id, observation.id)).expect("Unable to move file");
            }
        }

        if !has_next {
            break;
        }
    }

    Ok(())
}
