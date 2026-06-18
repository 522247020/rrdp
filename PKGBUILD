# Maintainer: Your Name <your@email.com>
# Contributor: Your Name <your@email.com>

pkgname=rrdp
pkgver=0.2.0
pkgrel=1
pkgdesc="xfreerdp3 的简洁命令行包装工具 / A CLI wrapper for xfreerdp3"
arch=('x86_64')
url="https://github.com/522247020/rrdp"
license=('MIT')
depends=('freerdp')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$srcdir/$pkgname-$pkgver"
    cargo build --release --frozen
}

package() {
    cd "$srcdir/$pkgname-$pkgver"
    install -Dm755 target/release/rrdp "$pkgdir/usr/bin/rrdp"
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}
