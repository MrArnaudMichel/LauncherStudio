# Desktop Entry Creator

A GTK4 desktop application for creating and managing `.desktop` files on Linux.

## Features
- GUI for creating `.desktop` launchers
- Supports all major desktop entry fields
- Localized fields and advanced options
- Saves to `~/.local/share/applications`

## Getting Started

### Prerequisites
- Rust (latest stable)
- GTK4 development libraries

### Build and Run
```bash
git clone https://github.com/yourusername/desktop_app.git
cd desktop_app
cargo build --release
./target/release/desktop_app
```

## Automatic .deb Packaging and Updates

### 1. Build .deb Package
Install cargo-deb:
```bash
cargo install cargo-deb
```
Build the package:
```bash
cargo deb -- --release
```
Your .deb file will be in `target/debian/`.

### 2. Create an APT Repository on GitHub Pages
- Create a `debian/` folder in your repository.
- Place your `.deb` files and a `Packages` index file there.
- Use [dpkg-scanpackages](https://manpages.debian.org/bullseye/dpkg-dev/dpkg-scanpackages.1.en.html) to generate the index:
  ```bash
  dpkg-scanpackages . /dev/null | gzip -9c > Packages.gz
  ```
- Enable GitHub Pages for your repo, set the source to the `debian/` folder.

### 3. Add AppStream Metadata
- Create a file like `fr.arnaudmichel.DesktopEntryCreator.metainfo.xml` in your repo (see [AppStream docs](https://www.freedesktop.org/software/appstream/docs/)).
- Place it in the `debian/` folder with your .deb.

### 4. User Installation and Updates
- Users add your repo to their sources:
  ```bash
  echo "deb [trusted=yes] https://yourusername.github.io/desktop_app/debian/ ./" | sudo tee /etc/apt/sources.list.d/desktop_app.list
  sudo apt update
  sudo apt install desktop-app
  ```
- When you publish a new .deb and update `Packages.gz`, users will see updates in GNOME Software or via `apt upgrade`.

## License
MIT

## Contributing
Pull requests are welcome. For major changes, open an issue first to discuss what you would like to change.

## Issues
Report bugs or feature requests via GitHub Issues.
