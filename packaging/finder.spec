Name:           finder
Version:        0.1.0
Release:        1%{?dist}
Summary:        A macOS Finder clone for Linux
License:        MIT
BuildArch:      x86_64

%description
A native macOS Finder clone built with GTK4 and Rust.

%prep
# Binary is pre-built

%build
# Binary is pre-built

%install
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/usr/share/applications
mkdir -p %{buildroot}/usr/share/icons/hicolor/scalable/apps
cp target/release/finder %{buildroot}/usr/bin/finder
cp data/finder.desktop %{buildroot}/usr/share/applications/finder.desktop
cp data/icons/hicolor/scalable/apps/finder.svg %{buildroot}/usr/share/icons/hicolor/scalable/apps/finder.svg

%files
/usr/bin/finder
/usr/share/applications/finder.desktop
/usr/share/icons/hicolor/scalable/apps/finder.svg

%changelog
* Mon Oct 12 2026 Andhika Kurniawanto <me@andikurnia.com> - 0.1.0-1
- Initial release
