# Maintainer: Your Name <your@email.com>
# Contributor: Your Name <your@email.com>

pkgname=rrdp
pkgver=0.1.0
pkgrel=1
pkgdesc="xfreerdp3 的简洁命令行包装工具 / A CLI wrapper for xfreerdp3"
arch=('x86_64')
url="https://github.com/522247020/rrdp"
license=('MIT')
depends=('freerdp')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('c479714b7436af74c9f4abe78134a04d2bf742cc8e869c22f267129edc556954')

build() {
    cd "$srcdir/$pkgname-$pkgver"
    cargo build --release --frozen
}

package() {
    cd "$srcdir/$pkgname-$pkgver"
    install -Dm755 target/release/rrdp "$pkgdir/usr/bin/rrdp"
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}