# Maintainer: Rin Me <https://github.com/rinme>
# Contributor: Jakub Szymański <jakubmateusz@poczta.onet.pl>

pkgname=woeusb-rs-git
_pkgname=woeusb-rs
pkgver=0.3.0.r0.gd19ac54
pkgrel=1
pkgdesc="A modern, safe, high-performance Linux application to create bootable Windows USB installers from ISO images or DVDs (Rust rewrite of WoeUSB)"
arch=('x86_64' 'aarch64')
url="https://github.com/rinme/woeusb-rs"
license=('GPL-3.0-or-later')
depends=(
    'gtk4'
    'libadwaita'
    'polkit'
    'parted'
    'util-linux'
    'dosfstools'
    'ntfs-3g'
    'grub'
    'p7zip'
)
makedepends=(
    'cargo'
    'rust'
    'git'
    'make'
    'pkgconf'
)
provides=('woeusb' 'woeusbgui' 'woeusb-ng' 'woeusb-rs')
conflicts=('woeusb' 'woeusbgui' 'woeusb-ng' 'woeusb-ng-git' 'woeusb-rs')

pkgver() {
    cd "$startdir"
    git describe --long --tags 2>/dev/null | sed 's/^v//;s/\([^-]*-g\)/r\1/;s/-/./g' || \
    printf "0.3.0.r%s.g%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
}

build() {
    cd "$startdir"
    export CARGO_TARGET_DIR="$startdir/target"
    make PREFIX=/usr build
}

check() {
    cd "$startdir"
    export CARGO_TARGET_DIR="$startdir/target"
    make test
}

package() {
    cd "$startdir"
    make DESTDIR="$pkgdir" PREFIX=/usr POLKIT_DIR=/usr/share/polkit-1/actions install
}
