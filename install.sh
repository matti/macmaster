#!/bin/sh
# Installs macmaster to ~/.local/bin and runs it:
#   curl -fsSL https://raw.githubusercontent.com/matti/macmaster/main/install.sh | sh
# Arguments are passed to macmaster, e.g. `| sh -s -- --dry-run`.
set -eu

base="https://github.com/matti/macmaster/releases/latest/download"
bin_dir="$HOME/.local/bin"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

echo "Ladataan macmaster..."
curl -fsSL "$base/macmaster" -o "$tmp/macmaster"
curl -fsSL "$base/macmaster.sha256" -o "$tmp/macmaster.sha256"
(cd "$tmp" && shasum -a 256 -c macmaster.sha256 >/dev/null) || {
	echo "Tarkistussumma ei täsmää, asennus keskeytetty." >&2
	exit 1
}

mkdir -p "$bin_dir"
install -m 755 "$tmp/macmaster" "$bin_dir/macmaster"
echo "Asennettu: $bin_dir/macmaster"

case ":$PATH:" in
*":$bin_dir:"*) ;;
*)
	# shellcheck disable=SC2016 # written literally into ~/.zprofile
	line='export PATH="$HOME/.local/bin:$PATH"'
	grep -qsxF "$line" "$HOME/.zprofile" || printf '\n%s\n' "$line" >>"$HOME/.zprofile"
	echo "Lisätty $bin_dir PATHiin (~/.zprofile), voimassa uusissa terminaaleissa."
	;;
esac

echo
"$bin_dir/macmaster" "$@"
