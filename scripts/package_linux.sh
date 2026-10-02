#!/usr/bin/env bash

set -euo pipefail

version=""
jar=""
out=""
epoch="1"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version) version="$2"; shift 2 ;;
    --jar) jar="$2"; shift 2 ;;
    --out) out="$2"; shift 2 ;;
    --epoch) epoch="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

[[ -n "$version" && -n "$jar" && -n "$out" ]] || {
  echo "usage: $0 --version X.Y.Z --jar <path> --out <dir> [--epoch N]" >&2
  exit 2
}
[[ -f "$jar" ]] || { echo "jar not found: $jar" >&2; exit 1; }

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
maintainer_name="Lampu"
maintainer_email="lampu@cord.lamp.delivery"
homepage="https://cord.lamp.delivery"
summary="Lampcord Discord client"
description="A Kotlin Multiplatform Material 3 Discord client."
license="GPL-3.0-or-later"

deb_requires=(
  java-runtime libgtk-3-0 libnotify4 libnss3 libxss1 libxtst6 xdg-utils
  libatspi2.0-0 libuuid1 libsecret-1-0
)
deb_recommends=(libappindicator3-1 webkit2gtk-4.1)
rpm_requires=(java-runtime gtk3 libnotify nss libXScrnSaver libXtst xdg-utils at-spi2-core libuuid libsecret)
pac_depends=(java-runtime gtk3 libnotify nss libxss libxtst xdg-utils at-spi2-core util-linux-libs libsecret)
pac_optdepends=("libappindicator-gtk3" "webkit2gtk-4.1: embedded web content")

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# An alpha appVersion (1.0.1-a3) has to be spelled differently for each packager: rpm and
# alpm both reject a hyphen outright, and Debian's hyphen means "revision", which would sort
# an alpha *after* the real release and make apt treat the release as a downgrade.
base_version="$version"
alpha=""
if [[ "$version" =~ ^(.+)-a([0-9]+)$ ]]; then
  base_version="${BASH_REMATCH[1]}"
  alpha="${BASH_REMATCH[2]}"
fi

# Debian: a tilde sorts before the plain version, which is the pre-release semantics we want.
deb_version="$base_version"
[[ -n "$alpha" ]] && deb_version="${base_version}~a${alpha}"

# RPM: base version in Version, alpha marker in Release, so 1.0.1-1.a3 sorts before 1.0.1-1.
rpm_version="$base_version"
rpm_release="$epoch"
[[ -n "$alpha" ]] && rpm_release="${epoch}.a${alpha}"

# alpm: alphanumerics only, so fold the marker into the version itself.
pac_version="$base_version"
[[ -n "$alpha" ]] && pac_version="${base_version}a${alpha}"

stage="$work/stage"
mkdir -p "$out" \
  "$stage/usr/lib/lampcord" \
  "$stage/usr/bin" \
  "$stage/usr/share/applications" \
  "$stage/usr/share/icons/hicolor/512x512/apps"

install -m 644 "$jar" "$stage/usr/lib/lampcord/lampcord.jar"
install -m 644 "$repo_root/packaging/aur/lampcord.desktop" "$stage/usr/share/applications/lampcord.desktop"
install -m 644 "$repo_root/packaging/aur/lampcord.png" \
  "$stage/usr/share/icons/hicolor/512x512/apps/lampcord.png"
install -m 755 "$repo_root/packaging/aur/lampcord-launcher" "$stage/usr/bin/lampcord"

# Debian -----------------------------------------------------------------------
deb_control="$stage/DEBIAN"
mkdir -p "$deb_control"
{
  echo "Package: lampcord"
  echo "Version: $deb_version"
  echo "Section: net"
  echo "Priority: optional"
  echo "Architecture: amd64"
  echo "Maintainer: $maintainer_name <$maintainer_email>"
  echo "Installed-Size: $(du -sk "$stage/usr/lib/lampcord" | cut -f1)"
  echo "Depends: $(IFS=, ; echo "${deb_requires[*]}")"
  echo "Recommends: $(IFS=, ; echo "${deb_recommends[*]}")"
  echo "Homepage: $homepage"
  echo "Description: $summary"
  echo " $description"
} > "$deb_control/control"

cat > "$deb_control/postinst" <<'POSTINST'
#!/bin/sh
set -e
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database -q /usr/share/applications || true
fi
POSTINST
chmod 755 "$deb_control/postinst"

deb_out="$out/lampcord-desktop-linux-x64-${version}.deb"
dpkg-deb --root-owner-group --build "$stage" "$deb_out" > /dev/null
rm -rf "$deb_control"
echo "built $deb_out"

# RPM -------------------------------------------------------------------------
rpm_version="$version"
rpm_release="$epoch"
if [[ "$version" =~ ^(.+)-a([0-9]+)$ ]]; then
  rpm_version="${BASH_REMATCH[1]}"
  rpm_release="${epoch}.a${BASH_REMATCH[2]}"
fi

rpm_stage="$work/rpm-stage"
cp -a "$stage" "$rpm_stage"
mkdir -p "$work/rpmbuild"/{SPECS,SOURCES,BUILD,RPMS,SRPMS,BUILDROOT}
{
  echo "Name:           lampcord"
  echo "Version:        $rpm_version"
  echo "Release:        $rpm_release"
  echo "Summary:        $summary"
  echo ""
  echo "License:        $license"
  echo "URL:            $homepage"
  echo "BuildArch:      x86_64"
  echo ""
  for dep in "${rpm_requires[@]}"; do echo "Requires:       $dep"; done
  echo ""
  echo "Recommends:     libappindicator-gtk3"
  echo "Recommends:     webkit2gtk-4.1"
  echo ""
  echo "%description"
  echo "$description"
  echo ""
  echo "%prep"
  echo ""
  echo "%build"
  echo ""
  echo "%install"
  echo "rm -rf %{buildroot}"
  echo "mkdir -p %{buildroot}"
  echo "cp -a %{_sourcedir}/rpm-stage/. %{buildroot}/"
  echo ""
  echo "%post"
  echo "%{_bindir}/update-desktop-database -q %{_datadir}/applications 2>/dev/null || :"
  echo ""
  echo "%postun"
  echo "%{_bindir}/update-desktop-database -q %{_datadir}/applications 2>/dev/null || :"
  echo ""
  echo "%files"
  echo "%{_bindir}/lampcord"
  echo "%{_datadir}/applications/lampcord.desktop"
  echo "%{_datadir}/icons/hicolor/512x512/apps/lampcord.png"
  echo "/usr/lib/lampcord/lampcord.jar"
} > "$work/rpmbuild/SPECS/lampcord.spec"

rpmbuild --define "_topdir $work/rpmbuild" --define "_sourcedir $work" -bb \
  "$work/rpmbuild/SPECS/lampcord.spec" > "$work/rpm.log" 2>&1 || {
  echo "rpmbuild failed:" >&2
  tail -20 "$work/rpm.log" >&2
  exit 1
}
built_rpm="$(find "$work/rpmbuild/RPMS" -name '*.rpm' -print -quit)"
[[ -n "$built_rpm" ]] || { echo "rpmbuild produced no rpm" >&2; exit 1; }
rpm_out="$out/lampcord-desktop-linux-x64-${version}.rpm"
mv "$built_rpm" "$rpm_out"
echo "built $rpm_out"

# pacman ----------------------------------------------------------------------
pac_root="$work/pac"
mkdir -p "$pac_root"
cp -a "$stage" "$pac_root/pkg"

{
  echo "pkgname = lampcord"
  echo "pkgbase = lampcord"
  echo "pkgver = $pac_version-$epoch"
  echo "pkgdesc = $description"
  echo "url = $homepage"
  echo "builddate = $(date +%s)"
  echo "packager = $maintainer_name <$maintainer_email>"
  echo "size = $(du -sb "$pac_root/pkg" | cut -f1)"
  echo "arch = x86_64"
  echo "license = $license"
  for dep in "${pac_depends[@]}"; do echo "depend = $dep"; done
  for opt in "${pac_optdepends[@]}"; do echo "optdepend = $opt"; done
  echo "provides = lampcord"
  # pacman has no 'conflicts' key; replaces is the equivalent for taking over the name.
  echo "replaces = lampcord"
  echo "backup = usr/share/applications/lampcord.desktop"
} > "$pac_root/pkg/.PKGINFO"

cat > "$pac_root/pkg/.INSTALL" <<'PACHOOK'
post_install() {
  if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications || true
  fi
}
post_upgrade() { post_install; }
PACHOOK

(cd "$pac_root/pkg" && bsdtar --format=mtree \
  --options='!all,use-set,type,uid,gid,mode,time,size,md5,sha256,link' \
  -czf .MTREE usr .PKGINFO .INSTALL)

mkdir -p "$out"
pac_out="$(cd "$out" && pwd)/lampcord-desktop-linux-x86_64-${version}.pkg.tar.xz"
(cd "$pac_root/pkg" && bsdtar -cJf "$pac_out" .PKGINFO .INSTALL .MTREE usr)
echo "built $pac_out"