# Sunrise Winds

Aurora borealis monitoring application.

- **Backend** (Rust): Polls NOAA SWPC data, computes aurora viewline, sends notifications
- **Frontend** (Vue + Vite): Interactive dashboard with map, Kp index, solar wind data

## Quick Start

### Backend

```bash
cd backend
cp config.example.toml config.toml  # edit with your location
cargo run
```

### Frontend

```bash
cd frontend
npm install
npm run dev
```

## Configuration

Copy `backend/config.example.toml` to `backend/config.toml` and edit your location coordinates and notification preferences.
