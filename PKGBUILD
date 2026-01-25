# Maintainer: Benjamin CH <contact@benjaminch.com>
pkgname=sobriquet
pkgver=0.1.0
pkgrel=1
pkgdesc="Fuzzy finder for shell aliases"
arch=('x86_64' 'aarch64')
url="https://github.com/benjaminch/sobriquet"
license=('MIT')
depends=()
makedepends=('cargo' 'rust')
options=('!lto')
source=("$pkgname-$pkgver.tar.gz::https://github.com/benjaminch/sobriquet/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('0000000000000000000000000000000000000000000000000000000000000000')

build() {
  cd "$pkgname-$pkgver"
  cargo build --release --locked
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm 755 "target/release/$pkgname" "$pkgdir/usr/local/bin/$pkgname"
  install -Dm 644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  install -Dm 644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"

  # Shell completions
  mkdir -p "$pkgdir/usr/share/zsh/site-functions"
  mkdir -p "$pkgdir/usr/share/bash-completion/completions"
  mkdir -p "$pkgdir/usr/share/fish/vendor_completions.d"

  "$pkgdir/usr/local/bin/$pkgname" generate complete-zsh > "$pkgdir/usr/share/zsh/site-functions/_$pkgname"
  "$pkgdir/usr/local/bin/$pkgname" generate complete-bash > "$pkgdir/usr/share/bash-completion/completions/$pkgname"
  "$pkgdir/usr/local/bin/$pkgname" generate complete-fish > "$pkgdir/usr/share/fish/vendor_completions.d/$pkgname.fish"

  # Man page
  mkdir -p "$pkgdir/usr/share/man/man1"
  "$pkgdir/usr/local/bin/$pkgname" generate man > "$pkgdir/usr/share/man/man1/$pkgname.1"
}
