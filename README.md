## Status

![CI](https://github.com/gofmanaa/weather/actions/workflows/ci.yml/badge.svg)
[![codecov](https://codecov.io/gh/gofmanaa/weather/branch/main/graph/badge.svg)](https://codecov.io/gh/gofmanaa/weather)
![Security Audit](https://github.com/gofmanaa/weather/actions/workflows/ci.yml/badge.svg?job=audit)
![Clippy](https://github.com/gofmanaa/weather/actions/workflows/ci.yml/badge.svg?job=test)

# weather

A fast, extensible, and modular command-line weather application written in Rust.

Supports multiple weather providers, including:

- OpenWeather
- WeatherAPI

### Features

- Fetch current weather data by city or coordinates
- Extensible provider system using Rust traits and generics
- Graceful handling of missing API keys (providers can be skipped)
- Supports `.env` files for API keys via `dotenvy`

### Installation

```bash
git clone https://github.com/gofmanaa/weather.git
cd weather-cli
cargo build --release
```

Or install directly via Cargo:

```bash
cargo install --git https://github.com/gofmanaa/weather.git
```

### Configuration

Before using the CLI, copy the example .env file and add your API keys:

```bash
cp env.example .env
```

Edit .env and set your keys:

```text
WEATHERAPI_API_KEY=PASTE_YOUR_API_KEY
OPENWEATHER_API_KEY=PASTE_YOUR_API_KEY
```

The CLI will automatically load these keys using dotenvy.

## Usage

### Configure default provider

```bash
weather configure <provider>
```

### Sets the default weather provider.

By default, the [WeatherApi](https://www.weatherapi.com/) provider is used.

Example:

```
weather configure openweather
```

### List supported providers

```bash
weather configure
```

Displays all available providers:

Available providers: ["weatherapi", "openweather"]

### Get weather

```bash
weather get <location> [date=now]
```

Fetches current or historical weather for a location.

Location can be in the format `city,country`.

Example:

```bash
weather get London,UK
weather get "New York,US" --date 2025-12-04
```

## Docker

```bash
docker buildx build --load -t weather .
docker run -it -v $(pwd)/.env:/app/.env --rm weather configure
docker run -it -v $(pwd)/.env:/app/.env --rm weather get Kharkiv,Ua
```

example output:

```bash
docker run -it -v $(pwd)/.env:/app/.env --rm weather get Kharkiv,Ua

Weather in Kharkiv,Ua: Sunny ☁️
> DateTeme: 2025-12-05 11:45,
> Temperature: 2.1°C,
> Humidity: 67.0 %,
> Pressure: 1028.0 hPa,
> Wind Speed: 10.4 k/h
> Wind Degree: 95.0°
Provider: WEATHERAPI
```