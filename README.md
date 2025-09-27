# Movie Vault

Movie Vault is a desktop application designed to help you manage and organize your personal movie and TV show collection. It automatically scans specified directories for media files, fetches comprehensive metadata from IMDb, and stores everything in a local SQLite database. The app provides a clean, intuitive interface to browse, filter, search, and manage your media library with features like tagging, infinite scrolling, and real-time file watching.

## Why Movie Vault?

In today's digital age, many people maintain large personal collections of movies and TV shows stored on their local drives. However, organizing and cataloging these collections can be challenging without proper tools. Movie Vault addresses this need by:

- **Automating Metadata Fetching**: Automatically retrieves detailed information from IMDb, including plot summaries, cast, ratings, genres, and more.
- **Local Storage**: Everything is stored locally in a SQLite database - no cloud dependency or subscription required.
- **Real-time Syncing**: Monitors your media directories for changes and updates the library automatically.
- **User-Friendly Interface**: Provides a modern, responsive UI for easy browsing and management.
- **Cross-Platform**: Built with Tauri, runs on Windows, macOS, and Linux.
- **Privacy-Focused**: All data remains on your device.

## Installation

### Option 1: Download Pre-built Binaries (Recommended)

Visit the [Releases](https://github.com/SAMoosavi/movie-vault/releases/latest) page and download the appropriate package for your operating system:

- **Linux**: `.rpm`, `.deb` or `.AppImage` files
- **Windows**: `.exe` installer
- **macOS**: `.dmg` bundle

Install the downloaded package using your system's package manager or installer.

### Option 2: Build from Source

#### Prerequisites

- [Node.js](https://nodejs.org/) (version 18 or higher)
- [pnpm](https://pnpm.io/) package manager
- [Rust](https://rustup.rs/) (latest stable version)

#### Steps

1. **Clone the repository:**

   ```bash
   git clone https://github.com/SAMoosavi/movie-vault.git
   cd movie-vault
   ```

2. **Install dependencies:**

   ```bash
   pnpm install
   ```

3. **Run in development mode:**

   ```bash
   pnpm tauri:dev
   ```

4. **Build for production:**
   ```bash
   pnpm build
   pnpm tauri:build
   ```

The built application will be available in the `src-tauri/target/release/bundle/` directory.

## Usage

### Getting Started

1. **Add Media Directories**: Go to settings or the initial setup to add folders containing your media files
2. **Automatic Syncing**: The app will scan your directories and fetch metadata from IMDb
3. **Browse Your Library**: Use the home page to view and filter your collection

### Key Features

- **Home Page**: Browse your media collection with card or list view, filter by various criteria
- **Media Details**: Click on any media item to view detailed information, cast, files, and tags
- **Add Media Manually**: Search IMDb or enter IMDb IDs directly to add specific titles
- **Tagging System**: Organize your media with custom tags
- **Settings**: Customize appearance (themes) and manage tags
- **Real-time Updates**: File changes in watched directories are automatically reflected

### Supported File Types

The app supports common video file formats including:

- .mp4, .mkv, .avi

## Technologies Used

### Frontend

- **Vue 3**: Progressive JavaScript framework with Composition API
- **TypeScript**: Type-safe JavaScript for better development experience
- **Tailwind CSS**: Utility-first CSS framework
- **DaisyUI**: Component library built on Tailwind CSS
- **Pinia**: State management library for Vue
- **Vue Router**: Official router for Vue.js
- **Lucide Vue Next**: Icon library

### Backend

- **Tauri**: Framework for building desktop apps with web technologies
- **Rust**: Systems programming language for the backend
- **SQLite**: Embedded database for local data storage
- **Diesel**: ORM and query builder for Rust

### Development Tools

- **Vite**: Fast build tool and development server
- **ESLint**: Code linting
- **Prettier**: Code formatting
- **Vue TSC**: TypeScript compiler for Vue

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
