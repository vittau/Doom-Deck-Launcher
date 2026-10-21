#!/usr/bin/env bash
# Generate the dedicated GPG key that signs the self-hosted Flatpak repository
# (spec 063). Run this once, on a machine you trust, then store the private key
# only as a GitHub Actions secret.
#
# Usage: scripts/flatpak/make-signing-key.sh [uid] [output-dir]
#
# The key is created in a throwaway GNUPGHOME, never in your default keyring.
# Nothing is uploaded; the script prints the exact `gh secret set` commands.
set -euo pipefail

uid="${1:-DoomDeckLauncher Flatpak repository <vitor.p.mach@gmail.com>}"
out_dir="${2:-$PWD/ddl-flatpak-signing-key}"

if ! command -v gpg >/dev/null 2>&1; then
  echo "error: gpg is not installed. On macOS: brew install gnupg" >&2
  exit 1
fi

if ! command -v base64 >/dev/null 2>&1; then
  echo "error: base64 is not installed" >&2
  exit 1
fi

if [ -e "$out_dir" ]; then
  echo "error: $out_dir already exists; refusing to overwrite key material" >&2
  exit 1
fi

umask 077
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

export GNUPGHOME="$work/gnupg"
install -d -m 0700 "$GNUPGHOME"

# rsa4096, sign-only, no expiry. Flatpak signs both the commit and the summary
# with it; clients pin the public half from the .flatpakref.
gpg --batch --quiet --pinentry-mode loopback --passphrase '' \
  --quick-generate-key "$uid" rsa4096 sign 0

fingerprint="$(gpg --batch --with-colons --list-secret-keys \
  | awk -F: '/^fpr:/ {print $10; exit}')"
if [ -z "$fingerprint" ]; then
  echo "error: the generated key has no fingerprint" >&2
  exit 1
fi

install -d "$out_dir"
gpg --batch --export "$fingerprint" > "$out_dir/ddl-flatpak-public.gpg"
gpg --batch --export-secret-keys --armor "$fingerprint" \
  | base64 | tr -d '\n' > "$out_dir/ddl-flatpak-secret.asc.b64"
base64 < "$out_dir/ddl-flatpak-public.gpg" \
  | tr -d '\n' > "$out_dir/ddl-flatpak-public.b64"
chmod 600 "$out_dir/ddl-flatpak-secret.asc.b64"

cat <<EOF
Generated the Flatpak repository key.

  Fingerprint: ${fingerprint}
  Public key:  ${out_dir}/ddl-flatpak-public.gpg
  Secret file: ${out_dir}/ddl-flatpak-secret.asc.b64 (mode 600, single line)

Next steps, in this order:

1. Store the key as a secret on the application repository:

     gh secret set FLATPAK_GPG_PRIVATE_KEY -R vittau/Doom-Deck-Launcher \\
       < "${out_dir}/ddl-flatpak-secret.asc.b64"
     gh secret set FLATPAK_GPG_KEY_ID -R vittau/Doom-Deck-Launcher \\
       --body "${fingerprint}"

2. Keep an offline backup of ${out_dir}/ddl-flatpak-secret.asc.b64
   (password manager or encrypted disk), then delete the local copy:

     rm -rf "${out_dir}"

   Losing the key means every existing user has to remove and re-add the
   remote to keep receiving updates.

3. The public key is embedded in the published .flatpakref and
   .flatpakrepo files; clients verify every commit and summary against it.
EOF
