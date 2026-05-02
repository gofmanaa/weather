use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex, RwLock, atomic::AtomicBool},
    time::Duration,
};

use chrono::{NaiveDateTime, Timelike, Utc};
use grib::{Grib2SubmessageDecoder, LatLons};
use tracing::{debug, info};

use crate::weather_providers::{WeatherData, WeatherProvider, error::ProviderError};

pub struct Grib2Provider {
    cache: Arc<RwLock<HashMap<String, WeatherData>>>,
    data_path: PathBuf,
    state: Arc<Mutex<AtomicBool>>,
}

impl Grib2Provider {
    pub fn new(data_path: Option<impl Into<PathBuf>>) -> Self {
        let data_path = match data_path {
            Some(p) => p.into(),
            None => PathBuf::from("./data"),
        };
        debug!("Grib2Provider data_path: {}", data_path.display());
        let provider = Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            data_path: data_path.into(),
            state: Arc::new(Mutex::new(AtomicBool::new(false))),
        };

        provider.start_background_updater();
        provider
    }

    fn start_background_updater(&self) {
        let data_path = self.data_path.clone();
        //let cache = self.cache.clone();

        debug!("Started start_background_updater");

        let is_ready = self.state.clone();
        tokio::spawn(async move {
            if let Err(e) = update_cycle(&data_path.display().to_string()).await {
                eprintln!("update error: {:?}", e);
            } else {
                is_ready
                    .lock()
                    .unwrap()
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
        });
    }
}

#[async_trait::async_trait]
impl WeatherProvider for Grib2Provider {
    async fn fetch(
        &self,
        location: &str,
        _date: Option<NaiveDateTime>,
    ) -> Result<WeatherData, ProviderError> {
        while !self
            .state
            .lock()
            .unwrap()
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            tokio::time::sleep(Duration::from_secs(10)).await;
            break;
        }
        info!("fetch");

        // 1. cache hit
        if let Some(w_data) = self.cache.read().unwrap().get(location) {
            return Ok(w_data.clone());
        }
        // 2. fallback - read directly

        let grib = format!(
            "{}{}.grib2",
            self.data_path.display(),
            build_enfo_file_name()
        );
        let index = format!(
            "{}{}.index",
            self.data_path.display(),
            build_enfo_file_name()
        );

        let loc = location.to_string();
        let data = tokio::task::spawn_blocking(move || extract_weather(&grib, &index, &loc))
            .await
            .map_err(ProviderError::Join)??;

        // 3. save cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(location.to_string(), data.clone());
        }

        Ok(data)
    }
}

async fn update_cycle(data_path: &str) -> Result<(), ProviderError> {
    let (grib_file, index_file) = download_latest(data_path).await?;

    if !std::path::Path::new(&index_file).exists() {
        debug!("index file not found");
        build_index(&grib_file, &index_file)?;
    }

    Ok(())
}

async fn download_latest(data_path: &str) -> Result<(String, String), ProviderError> {
    let date = chrono::Utc::now().format("%Y%m%d").to_string();
    let run = "00z";

    let base = format!("s3://ecmwf-forecasts/{}/{}/aifs-ens/0p25/enfo/", date, run);

    let grib_file = format!("{}{}.grib2", data_path, build_enfo_file_name());
    let index_file = format!("{}{}.index", data_path, build_enfo_file_name());

    let grib_s3 = format!("{base}{}.grib2", build_enfo_file_name());
    let index_s3 = format!("{base}{}.index", build_enfo_file_name());

    let grib_clone = grib_file.clone();
    let index_clone = index_file.clone();

    tokio::task::spawn_blocking(move || -> Result<(), ProviderError> {
        if !PathBuf::from(index_clone.clone()).exists() {
            debug!("Download index file from S3: {}", index_s3);
            let index_status = Command::new("aws")
                .args(["s3", "cp", "--no-sign-request", &index_s3, &index_clone])
                .status()
                .map_err(ProviderError::Io)?;

            if !index_status.success() {
                return Err(ProviderError::Download("index download failed".into()));
            }
        }

        if !PathBuf::from(grib_clone.clone()).exists() {
            debug!("Download grib2 file from S3: {}", grib_s3);
            let status = Command::new("aws")
                .args(["s3", "cp", "--no-sign-request", &grib_s3, &grib_clone])
                .status()
                .map_err(ProviderError::Io)?;

            if !status.success() {
                return Err(ProviderError::Download(
                    "grib download failed after retries".into(),
                ));
            }
        }

        Ok(())
    })
    .await
    .map_err(|e| ProviderError::Join(e))??;

    Ok((grib_file, index_file))
}

fn build_index(grib: &str, index: &str) -> Result<(), ProviderError> {
    let output = std::process::Command::new("grib_ls")
        .arg("-j")
        .arg(grib)
        .output()
        .map_err(ProviderError::Io)?;
    debug!("build index");
    std::fs::write(index, output.stdout).map_err(ProviderError::Io)?;

    Ok(())
}

fn extract_weather(
    grib_path: &str,
    index_path: &str,
    location: &str,
) -> Result<WeatherData, ProviderError> {
    let (lat, lon) = geocode(location);
    debug!("Extract weather data");
    let index = load_index(index_path)?;

    let mut file = std::fs::File::open(grib_path)
        .map_err(ProviderError::Io)
        .expect("error open file grib2");
    debug!("grib2 file open");
    // 🔥 читаємо параметри
    let t = decode_value_at(
        read_grib_chunk(&mut file, index["2t"]._offset, index["2t"]._length)?,
        lat,
        lon,
    )?;

    debug!("temperature: {}", t);

    let td = decode_value_at(
        read_grib_chunk(&mut file, index["2d"]._offset, index["2d"]._length)?,
        lat,
        lon,
    )?;

    let sp = decode_value_at(
        read_grib_chunk(&mut file, index["msl"]._offset, index["msl"]._length)?,
        lat,
        lon,
    )?;

    let u = decode_value_at(
        read_grib_chunk(&mut file, index["10u"]._offset, index["10u"]._length)?,
        lat,
        lon,
    )?;

    let v = decode_value_at(
        read_grib_chunk(&mut file, index["10v"]._offset, index["10v"]._length)?,
        lat,
        lon,
    )?;

    let tcc = decode_value_at(
        read_grib_chunk(&mut file, index["tcc"]._offset, index["tcc"]._length)?,
        lat,
        lon,
    )?;

    // 🌡 temperature
    let temp_c = t - 273.15;
    let td_c = td - 273.15;

    // 💧 humidity
    let humidity = 100.0 * ((17.625 * td_c) / (243.04 + td_c)).exp()
        / ((17.625 * temp_c) / (243.04 + temp_c)).exp();

    // 🌬 wind
    let wind_speed = (u * u + v * v).sqrt() * 3.6;
    let wind_deg = v.atan2(u).to_degrees();

    // ☁ condition
    let condition = if tcc < 0.2 {
        "Clear"
    } else if tcc < 0.5 {
        "Partly Cloudy"
    } else {
        "Cloudy"
    };

    Ok(WeatherData {
        location: location.to_string(),
        datetime: Utc::now(),
        temp_c: temp_c.into(),
        humidity: humidity.into(),
        pressure: (sp / 100.0) as f64,
        condition: condition.into(),
        wind_kph: wind_speed.into(),
        wind_deg: wind_deg.into(),
    })
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
struct IndexRecord {
    param: String,
    levtype: String,
    #[serde(default)]
    levelist: Option<String>,
    _offset: u64,
    _length: u64,
}

fn load_index(index_path: &str) -> Result<HashMap<String, IndexRecord>, ProviderError> {
    debug!("load index: {}", index_path);
    let content = std::fs::read_to_string(index_path).map_err(ProviderError::Io)?;

    let mut map = HashMap::new();

    for line in content.lines() {
        let rec: IndexRecord =
            serde_json::from_str(line).map_err(|e| ProviderError::Parse(e.to_string()))?;

        if rec.levtype == "sfc" {
            map.insert(rec.param.clone(), rec);
        }
    }

    Ok(map)
}

fn read_grib_chunk(
    file: &mut std::fs::File,
    offset: u64,
    length: u64,
) -> Result<Vec<u8>, ProviderError> {
    let mut buf = vec![0u8; length as usize];
    file.seek(SeekFrom::Start(offset))
        .map_err(ProviderError::Io)?;
    file.read_exact(&mut buf).map_err(ProviderError::Io)?;

    Ok(buf)
}

fn decode_value_at(bytes: Vec<u8>, target_lat: f32, target_lon: f32) -> Result<f32, ProviderError> {
    use std::io::Cursor;

    let reader = Cursor::new(bytes);

    let grib = grib::from_reader(reader).map_err(|e| ProviderError::Parse(e.to_string()))?;

    let (_, sm) = grib.iter().next().ok_or(ProviderError::NotFound)?;

    let latlons = sm
        .latlons()
        .map_err(|e| ProviderError::Parse(e.to_string()))?;

    let decoder =
        Grib2SubmessageDecoder::from(sm).map_err(|e| ProviderError::Parse(e.to_string()))?;

    let values = decoder
        .dispatch()
        .map_err(|e| ProviderError::Parse(e.to_string()))?;

    let mut best = None;
    let mut best_dist = f32::MAX;

    for ((lat, lon), value) in latlons.zip(values) {
        let lon = if lon < 0.0 { lon + 360.0 } else { lon };

        let dlat = lat - target_lat;
        let dlon = lon - target_lon;

        let dist = dlat * dlat + dlon * dlon;

        if dist < best_dist {
            best_dist = dist;
            best = Some(value);
        }
    }

    best.ok_or(ProviderError::NotFound)
}

fn geocode(location: &str) -> (f32, f32) {
    match location {
        "Portugal, Porto" => (41.15, 351.25),
        _ => (41.15, 351.25),
    }
}

fn build_enfo_file_name() -> String {
    let now = Utc::now();
    let hour = now.hour();
    let run = match hour {
        0..=5 => "6h",
        6..=11 => "12h",
        12..=17 => "18h",
        _ => "0h",
    };
    let date_str = now.format("%Y%m%d").to_string();

    format!("{}000000-{}-enfo-cf", date_str, run)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_name() {
        // s3://ecmwf-forecasts/20260502/00z/aifs-ens/0p25/enfo/
        // 2026-05-02 06:55:03   84945948 20260502000000-18h-enfo-cf.grib2
        // 2026-05-02 06:55:03      23662 20260502000000-18h-enfo-cf.index
        //
        let f = build_enfo_file_name();
        dbg!(f);
    }
}
