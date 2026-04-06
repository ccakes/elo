# Maintainer: cdaniel
pkgname=elo
pkgver=0.2.3
pkgrel=1
pkgdesc='Elo — a Numi-compatible calculator'
arch=('x86_64')
url='https://github.com/ccakes/elo'
license=('MIT')
depends=('libayatana-appindicator' 'webkit2gtk-4.1' 'gtk3')
makedepends=('curl' 'jq')
source=("Elo_${pkgver}_amd64.deb::${url}/releases/download/v${pkgver}/Elo_${pkgver}_amd64.deb")
sha256sums=('SKIP')
noextract=("Elo_${pkgver}_amd64.deb")

pkgver() {
  curl -s https://api.github.com/repos/ccakes/elo/releases/latest \
    | jq -r '.tag_name' \
    | sed 's/^v//'
}

prepare() {
  cd "$srcdir"
  ar x "Elo_${pkgver}_amd64.deb"
  tar xzf data.tar.gz
}

package() {
  cp -a "$srcdir/usr" "$pkgdir/usr"
}
