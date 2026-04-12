# Sabot TB Clinic Management System

[![Tauri](https://img.shields.io/badge/Tauri-%5E2.0.0-24c8db?logo=tauri&logoColor=fff)](https://tauri.app/)
[![Vue.js](https://img.shields.io/badge/Vue.js-%5E3.5.0-4FC08D?logo=vuedotjs&logoColor=fff)](https://vuejs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-%5E5.4.0-3178C6?logo=typescript&logoColor=ffffff)](https://www.typescriptlang.org/)
[![Vite](https://img.shields.io/badge/Vite-%5E5.0.0-646CFF?logo=vite&logoColor=ffffff)](https://vitejs.dev/)
[![Pinia](https://img.shields.io/badge/Pinia-%5E2.1.7-FFE16B?logo=vue.js&logoColor=black)](https://pinia.vuejs.org/)
[![Vue_Router](https://img.shields.io/badge/Vue_Router-%5E4.3.0-4FC08D?logo=vuedotjs&logoColor=ffffff)](https://router.vuejs.org/)
[![Lucide](https://img.shields.io/badge/Lucide-%5E0.400.0-F92672?logo=lucide&logoColor=ffffff)](https://lucide.dev/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A modern desktop application built with Tauri (Rust) and Vue (TypeScript) for managing tuberculosis (TB) clinic operations at Sasot Hospital. This system bridges HOSxP's MySQL database (read-only) with a local SQLite database for clinic-specific tracking data.

## Features

- **Drug Screening**: Query HOSxP patients who have received TB drugs to identify new patients.
- **Active Patients Dashboard**: Overview of currently active TB patients with treatment progress and an alert engine tracking patient adherence.
- **Patient Details**: Full clinical timeline, drug dispensing history, monthly follow-up records, and side-effect tracking.
- **Reports**: Summary statistics and exportable CSV reports for TB clinic audits (HA Standard).
- **Settings**: Complete control to configure database connections, update TB drug codes, and export local SQLite backups.

## Architecture

- **Frontend**: Vue 3.5 (Composition API, `<script setup>`), TypeScript 5, Pinia, Vue Router 4, and `lucide-vue-next` for iconography.
- **Backend / Desktop**: Tauri 2.5, Rust (stable), and `sqlx` driving database interactions.
- **Databases**:
  - **MySQL (HOSxP)**: Read-only access to query patient demographics and their drug dispensing records.
  - **SQLite**: Local, read-write database holding clinic metrics, treatment plans, follow-ups, and statuses.

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (Version 18+)
- [Rust](https://www.rust-lang.org/)
- Platform [Prerequisites for Tauri 2](https://v2.tauri.app/start/prerequisites/)

### Setup

1. **Clone the repository** (and switch into the project directory):

   ```bash
   git clone https://github.com/pharmacist-sabot/sabot-tb-clinic.git
   cd sabot-tb-clinic
   ```

2. **Install frontend dependencies**:

   ```bash
   npm install
   ```

3. **Run the development server**:

   ```bash
   npm run tauri dev
   ```

4. **Build for release** (generates OS specific execution binary):

   ```bash
   npm run tauri build
   ```

## License

This project is licensed under the [MIT License](LICENSE).
