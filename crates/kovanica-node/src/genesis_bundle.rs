//! Deterministic genesis bundle writer.
//!
//! A genesis bundle is the file set every node on a network must hold byte for
//! byte: `genesis_validators.toml` plus, depending on the chain,
//! `founder_premine.hex`, `funded_accounts.toml` and `validators.json`. Every
//! one of them is an input to [`crate::genesis_config_digest`], so a single
//! byte that differs on one node changes its network tag and leaves that node
//! with no peers.
//!
//! The archive is written deterministically: entries are sorted, every tar
//! header field that could vary (mtime, uid, gid, owner names, mode) is fixed,
//! and the gzip layer stores the deflate stream uncompressed. Writing the
//! container by hand instead of through a compression crate keeps the bytes
//! independent of the host's zlib version and of a dependency's release
//! history — the same files always produce the same archive on every machine.
//! A genesis file set is a few kilobytes, so compression would save nothing.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};

/// Files a bundle may contain. Everything else in the source directory is
/// refused: a bundle is published, so a stray key, keystore or `config.toml`
/// must never be swept into one.
pub(crate) const GENESIS_FILES: &[&str] = &[
    "genesis_validators.toml",
    "founder_premine.hex",
    "funded_accounts.toml",
    "validators.json",
];

/// The one file a bundle cannot do without: it declares the chain id and the
/// emission policy, and `node/scripts/install.sh` refuses a bundle without it.
pub(crate) const REQUIRED_FILE: &str = "genesis_validators.toml";

/// Shipped inside the bundle: the tag `node/scripts/install.sh` compares the
/// node's tag against, so a release can add a network without editing it.
pub(crate) const TAG_FILE: &str = "expected-tag";

/// Shipped inside the bundle: sha256 of each genesis file, in the format
/// `sha256sum -c` verifies.
pub(crate) const SUMS_FILE: &str = "SHA256SUMS";

/// A built bundle, not yet written to disk.
#[derive(Debug)]
pub(crate) struct Bundle {
    /// Genesis files included, sorted by name.
    pub(crate) genesis_files: Vec<(String, Vec<u8>)>,
    /// The complete `.tar.gz` bytes.
    pub(crate) archive: Vec<u8>,
}

/// Read the genesis files from `source` and build the archive.
///
/// `tag` is the network tag the caller resolved for these exact files; it is
/// written into the bundle as [`TAG_FILE`]. Nothing in this module touches the
/// network or a database.
pub(crate) fn build(source: &Path, chain_id: u64, tag: &str) -> Result<Bundle> {
    let genesis_files = read_genesis_files(source, chain_id)?;

    let mut sums = String::new();
    for (name, bytes) in &genesis_files {
        sums.push_str(&sha256_hex(bytes));
        sums.push_str("  ");
        sums.push_str(name);
        sums.push('\n');
    }

    let mut entries = genesis_files.clone();
    entries.push((SUMS_FILE.to_string(), sums.into_bytes()));
    entries.push((TAG_FILE.to_string(), format!("{tag}\n").into_bytes()));
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(Bundle {
        genesis_files,
        archive: gzip_stored(&tar_ustar(&entries)),
    })
}

/// Read and validate the genesis file set in `source`.
fn read_genesis_files(source: &Path, chain_id: u64) -> Result<Vec<(String, Vec<u8>)>> {
    if !source.is_dir() {
        bail!(
            "--source {} is not a directory; it must hold the genesis files \
             ({REQUIRED_FILE} at least)",
            source.display()
        );
    }

    let mut found: Vec<(String, PathBuf)> = Vec::new();
    let mut refused: Vec<String> = Vec::new();
    let listing = std::fs::read_dir(source)
        .with_context(|| format!("reading --source {}", source.display()))?;
    for entry in listing {
        let entry = entry.with_context(|| format!("reading --source {}", source.display()))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();
        if !GENESIS_FILES.contains(&name.as_str()) {
            refused.push(name);
        } else if !path.is_file() {
            bail!(
                "{} is not a regular file; a genesis file cannot be a directory or a symlink",
                path.display()
            );
        } else {
            found.push((name, path));
        }
    }

    if !refused.is_empty() {
        refused.sort();
        bail!(
            "--source {} also contains {}; a bundle is published, so keys, keystores, \
             configs and data directories must stay out of the source directory. \
             A bundle may only contain: {}",
            source.display(),
            refused.join(", "),
            GENESIS_FILES.join(", ")
        );
    }

    if !found.iter().any(|(name, _)| name == REQUIRED_FILE) {
        bail!(
            "--source {} has no {REQUIRED_FILE}; it is the file that declares the chain id \
             and the emission policy, and every node needs it",
            source.display()
        );
    }

    if !crate::funded_accounts_allowed(chain_id)
        && found.iter().any(|(n, _)| n == "funded_accounts.toml")
    {
        bail!(
            "--source {} contains funded_accounts.toml, which chain {chain_id} (mainnet) \
             forbids: every unit of the supply comes from the published allocation, not \
             from a file. Remove it before bundling",
            source.display()
        );
    }

    found.sort_by(|a, b| a.0.cmp(&b.0));
    let mut files = Vec::with_capacity(found.len());
    for (name, path) in found {
        let bytes = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
        files.push((name, bytes));
    }
    Ok(files)
}

/// Hex-encoded SHA-256 of `bytes`.
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

// --------------------------------------------------------------------------
// tar (POSIX ustar)
// --------------------------------------------------------------------------

const BLOCK: usize = 512;

/// Serialise `entries` as a ustar archive with fixed, host-independent
/// metadata.
fn tar_ustar(entries: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    for (name, data) in entries {
        append_tar_entry(&mut out, name, data);
    }
    // An archive ends with two zero blocks.
    out.extend_from_slice(&[0u8; 2 * BLOCK]);
    out
}

fn append_tar_entry(out: &mut Vec<u8>, name: &str, data: &[u8]) {
    debug_assert!(!name.contains('/'), "entries are top-level files only");
    debug_assert!(
        name.len() <= 100,
        "tar `name` field holds at most 100 bytes"
    );

    let mut header = [0u8; BLOCK];
    header[..name.len()].copy_from_slice(name.as_bytes());
    // mode 0644, uid 0, gid 0 — the archive must not depend on who built it.
    header[100..108].copy_from_slice(b"0000644\0");
    header[108..116].copy_from_slice(b"0000000\0");
    header[116..124].copy_from_slice(b"0000000\0");
    // Octal `size`, 11 digits plus NUL.
    let size = format!("{:011o}\0", data.len());
    header[124..136].copy_from_slice(size.as_bytes());
    // mtime 0 (1970-01-01T00:00:00Z), so a rebuild is byte-identical.
    header[136..148].copy_from_slice(b"00000000000\0");
    // Checksum field is spaces while the sum is computed.
    header[148..156].copy_from_slice(b"        ");
    header[156] = b'0'; // regular file
    header[257..263].copy_from_slice(b"ustar\0");
    header[263..265].copy_from_slice(b"00");
    // uname / gname / devmajor / devminor stay zero: no owner, no device.

    let sum: u32 = header.iter().map(|byte| u32::from(*byte)).sum();
    let checksum = format!("{sum:06o}\0 ");
    header[148..156].copy_from_slice(checksum.as_bytes());

    out.extend_from_slice(&header);
    out.extend_from_slice(data);
    let padding = (BLOCK - data.len() % BLOCK) % BLOCK;
    out.extend(std::iter::repeat_n(0u8, padding));
}

// --------------------------------------------------------------------------
// gzip (RFC 1952) with stored (RFC 1951 §3.2.4) blocks
// --------------------------------------------------------------------------

/// Wrap `data` in a gzip container whose deflate stream is a sequence of
/// stored blocks.
///
/// Nothing is compressed, which is deliberate: DEFLATE output depends on the
/// encoder and its version, so compressing would make the archive bytes depend
/// on the build host. `tar -xzf`, `gzip -d` and every other reader accept
/// stored blocks.
fn gzip_stored(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + 2 * BLOCK + data.len() / 65_535 * 5 + 18);
    // ID1, ID2, CM=deflate, FLG=0 (no name, no comment), MTIME=0, XFL=0,
    // OS=255 (unknown).
    out.extend_from_slice(&[0x1f, 0x8b, 0x08, 0x00, 0, 0, 0, 0, 0x00, 0xff]);

    let mut chunks = data.chunks(65_535).peekable();
    if chunks.peek().is_none() {
        // A final stored block of length 0: LEN = 0, NLEN = 0xffff.
        out.extend_from_slice(&[0x01, 0x00, 0x00, 0xff, 0xff]);
    }
    while let Some(chunk) = chunks.next() {
        let last = chunks.peek().is_none();
        // BFINAL in bit 0, BTYPE=00 (stored) in bits 1-2, rest of the byte
        // padded to the byte boundary.
        out.push(u8::from(last));
        let len = u16::try_from(chunk.len()).expect("chunk is at most 65535 bytes");
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(chunk);
    }

    out.extend_from_slice(&crc32(data).to_le_bytes());
    out.extend_from_slice(&u32::try_from(data.len()).unwrap_or(u32::MAX).to_le_bytes());
    out
}

/// CRC-32 (IEEE 802.3, reflected, polynomial `0xEDB88320`), the checksum gzip
/// trailers carry.
fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in data {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    fn write(dir: &Path, name: &str, contents: &[u8]) {
        std::fs::write(dir.join(name), contents).expect("write fixture");
    }

    // -- the container ------------------------------------------------------

    #[test]
    fn crc32_matches_the_reference_vector() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
        assert_eq!(crc32(b""), 0);
    }

    /// Independent reader: walks the stored blocks and checks the trailer.
    fn inflate_stored(gz: &[u8]) -> Vec<u8> {
        assert_eq!(&gz[..3], &[0x1f, 0x8b, 0x08], "gzip magic and CM=deflate");
        assert_eq!(gz[3], 0x00, "no optional gzip fields");
        assert_eq!(&gz[4..8], &[0, 0, 0, 0], "mtime is zero");
        assert_eq!(gz[9], 255, "OS unknown");

        let mut at = 10;
        let mut out = Vec::new();
        loop {
            let bfinal = gz[at] & 1;
            let btype = (gz[at] >> 1) & 0b11;
            assert_eq!(btype, 0, "every block is stored");
            at += 1;
            let len = u16::from_le_bytes([gz[at], gz[at + 1]]);
            let nlen = u16::from_le_bytes([gz[at + 2], gz[at + 3]]);
            assert_eq!(!len, nlen, "NLEN is the complement of LEN");
            at += 4;
            out.extend_from_slice(&gz[at..at + usize::from(len)]);
            at += usize::from(len);
            if bfinal == 1 {
                break;
            }
        }
        let crc = u32::from_le_bytes([gz[at], gz[at + 1], gz[at + 2], gz[at + 3]]);
        let isize = u32::from_le_bytes([gz[at + 4], gz[at + 5], gz[at + 6], gz[at + 7]]);
        assert_eq!(crc, crc32(&out), "trailer CRC");
        assert_eq!(usize::try_from(isize).unwrap(), out.len(), "trailer size");
        assert_eq!(at + 8, gz.len(), "no trailing bytes");
        out
    }

    /// Independent reader: parses ustar headers and verifies their checksums.
    fn read_tar(tar: &[u8]) -> Vec<(String, Vec<u8>)> {
        let mut entries = Vec::new();
        let mut at = 0;
        loop {
            let header = &tar[at..at + BLOCK];
            if header.iter().all(|byte| *byte == 0) {
                assert_eq!(
                    &tar[at + BLOCK..at + 2 * BLOCK],
                    &[0u8; BLOCK],
                    "the archive ends with two zero blocks"
                );
                assert_eq!(at + 2 * BLOCK, tar.len(), "nothing after the final blocks");
                return entries;
            }

            let name = String::from_utf8(
                header[..100]
                    .iter()
                    .take_while(|byte| **byte != 0)
                    .copied()
                    .collect(),
            )
            .expect("utf-8 name");
            assert_eq!(header[156], b'0', "regular file");
            assert_eq!(&header[257..265], b"ustar\x0000", "ustar magic");

            let field = |range: std::ops::Range<usize>| -> u64 {
                let raw = &header[range.clone()];
                let text = std::str::from_utf8(raw).expect("ascii field");
                u64::from_str_radix(text.trim_end_matches(['\0', ' ']), 8)
                    .unwrap_or_else(|e| panic!("octal field {raw:?}: {e}"))
            };
            assert_eq!(field(100..108), 0o644, "mode");
            assert_eq!(field(108..116), 0, "uid");
            assert_eq!(field(116..124), 0, "gid");
            assert_eq!(field(136..148), 0, "mtime");

            let mut checksum_header = [0u8; BLOCK];
            checksum_header.copy_from_slice(header);
            checksum_header[148..156].copy_from_slice(b"        ");
            let expected: u32 = checksum_header.iter().map(|byte| u32::from(*byte)).sum();
            assert_eq!(field(148..156), u64::from(expected), "header checksum");

            let size = usize::try_from(field(124..136)).expect("size fits");
            at += BLOCK;
            entries.push((name, tar[at..at + size].to_vec()));
            at += size;
            at += (BLOCK - size % BLOCK) % BLOCK;
        }
    }

    #[test]
    fn archive_round_trips_through_independent_readers() {
        let tar = tar_ustar(&[
            ("a.toml".to_string(), b"chain_id = 2\n".to_vec()),
            ("b.hex".to_string(), Vec::new()),
            ("c.json".to_string(), vec![b'x'; 4096]),
        ]);
        let gz = gzip_stored(&tar);
        assert_eq!(inflate_stored(&gz), tar);

        let entries = read_tar(&tar);
        let names: Vec<&str> = entries.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(names, ["a.toml", "b.hex", "c.json"]);
        assert_eq!(entries[0].1, b"chain_id = 2\n");
        assert!(entries[1].1.is_empty());
        assert_eq!(entries[2].1.len(), 4096);
    }

    #[test]
    fn archive_handles_an_empty_payload_and_large_ones() {
        let gz = gzip_stored(&[]);
        assert!(inflate_stored(&gz).is_empty());

        // More than one stored block: 200_000 bytes is four blocks.
        let payload = vec![7u8; 200_000];
        let gz = gzip_stored(&payload);
        assert!(gz.len() > payload.len(), "stored blocks grow the archive");
        assert_eq!(inflate_stored(&gz), payload);
    }

    // -- the source directory ----------------------------------------------

    #[test]
    fn bundle_is_deterministic_and_sorted() {
        let dir = temp_dir();
        write(dir.path(), "genesis_validators.toml", b"chain_id = 2\n");
        write(dir.path(), "founder_premine.hex", b"aa\n");
        write(dir.path(), "funded_accounts.toml", b"[[account]]\n");

        let first = build(dir.path(), 2, "2-0123456789abcdef").expect("build");
        let second = build(dir.path(), 2, "2-0123456789abcdef").expect("build");
        assert_eq!(first.archive, second.archive, "same inputs, same bytes");

        let names: Vec<&str> = first
            .genesis_files
            .iter()
            .map(|(name, _)| name.as_str())
            .collect();
        assert_eq!(
            names,
            [
                "founder_premine.hex",
                "funded_accounts.toml",
                "genesis_validators.toml"
            ]
        );

        // The archive carries the file set plus the two files the installer
        // and the operator need, in byte order.
        let entries = read_tar(&inflate_stored(&first.archive));
        let names: Vec<&str> = entries.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(
            names,
            [
                "SHA256SUMS",
                "expected-tag",
                "founder_premine.hex",
                "funded_accounts.toml",
                "genesis_validators.toml"
            ]
        );
        let tag = entries
            .iter()
            .find(|(name, _)| name == TAG_FILE)
            .expect("expected-tag");
        assert_eq!(tag.1, b"2-0123456789abcdef\n");
    }

    #[test]
    fn sums_cover_the_genesis_files_only() {
        let dir = temp_dir();
        write(dir.path(), "genesis_validators.toml", b"chain_id = 2\n");
        write(dir.path(), "founder_premine.hex", b"aa\n");

        let bundle = build(dir.path(), 2, "2-0123456789abcdef").expect("build");
        let entries = read_tar(&inflate_stored(&bundle.archive));
        let sums = entries
            .iter()
            .find(|(name, _)| name == SUMS_FILE)
            .expect("SHA256SUMS");

        // Exactly the format `sha256sum -c` reads, over the genesis files and
        // nothing else (not the bundle's own metadata).
        assert_eq!(
            String::from_utf8(sums.1.clone()).expect("utf-8"),
            format!(
                "{}  founder_premine.hex\n{}  genesis_validators.toml\n",
                sha256_hex(b"aa\n"),
                sha256_hex(b"chain_id = 2\n")
            )
        );
    }

    #[test]
    fn unknown_files_in_the_source_are_refused() {
        let dir = temp_dir();
        write(dir.path(), "genesis_validators.toml", b"chain_id = 2\n");
        write(dir.path(), "node1_signing.key", b"secret\n");

        let err = build(dir.path(), 2, "2-0123456789abcdef").expect_err("refused");
        let message = err.to_string();
        assert!(message.contains("node1_signing.key"), "got: {message}");
        assert!(message.contains("must stay out"), "got: {message}");
    }

    #[test]
    fn a_bundle_without_genesis_validators_is_refused() {
        let dir = temp_dir();
        write(dir.path(), "founder_premine.hex", b"aa\n");

        let err = build(dir.path(), 2, "2-0123456789abcdef").expect_err("refused");
        assert!(
            err.to_string().contains("has no genesis_validators.toml"),
            "got: {err}"
        );
    }

    #[test]
    fn mainnet_refuses_funded_accounts() {
        let dir = temp_dir();
        write(dir.path(), "genesis_validators.toml", b"chain_id = 1\n");
        write(dir.path(), "funded_accounts.toml", b"[[account]]\n");

        let err = build(dir.path(), 1, "1-0123456789abcdef").expect_err("refused");
        assert!(err.to_string().contains("mainnet) forbids"), "got: {err}");
    }

    #[test]
    fn a_missing_source_directory_is_refused() {
        let dir = temp_dir();
        let err = build(&dir.path().join("nope"), 2, "2-0123456789abcdef").expect_err("refused");
        assert!(err.to_string().contains("is not a directory"), "got: {err}");
    }
}
