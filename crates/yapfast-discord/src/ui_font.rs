//! gg sans, the typeface of Discord's apps. Its licence doesn't allow passing
//! it on, so Yapfast doesn't ship it: it downloads the web font files Discord's
//! own web client loads, from Discord, once, and keeps them in its data folder
//! as TrueType files. Until they're there, Yapfast draws its own typeface.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};
use sha2::{Digest, Sha256};

/// The family name the files carry.
pub const FAMILY: &str = "gg sans";

/// The folder under the data folder that holds the files.
const FOLDER: &str = "fonts";

/// Each style: the web client's file under `discord.com/assets/`, the name
/// Yapfast keeps it under, and the SHA-256 of the WOFF2 file Discord serves.
/// Discord's asset names change with their content, so a new version of the
/// font comes with new names, and the old files stay where they were.
const STYLES: [(&str, &str, &str); 10] = [
    ("66d715454104d24e", "ggsans-Regular", "8e401b056dc1eb48d44a01407ceb54372bbc44797d3259069ce96a96dfd8c104"),
    ("dd24010f3cf7def7", "ggsans-Italic", "b78fb569265b01789e7edd88cfe02ecb2c3fee5e1999678255f9b78a3b2cc4e8"),
    ("b272b33815319bae", "ggsans-Medium", "440ad8b1449985479bc37265e9912bbf2bf56fe9ffd14709358a8e9c2d5f8e5b"),
    ("6a1346ad3821ff3c", "ggsans-MediumItalic", "8957b06e2baed65915fa19cdc3fb3dc48b9e94898b922674f6b7a1875199f466"),
    ("2df2c3ff74408972", "ggsans-Semibold", "ae99aaede2f373187a4fe442a2cb0ab9c2945efbab01cf33e01be517c0c4f813"),
    ("d5d789aeb6282532", "ggsans-SemiboldItalic", "f7e3571c1b8df4df3279a577718e545289a89501fcd0073bebbee8df7e8a06c7"),
    ("189422196a4f8b53", "ggsans-Bold", "13da02ce62b1a388a7c8d6f3bd286fe774ee2b91ac63d281523e80b2a8a063bc"),
    ("ce3b8055f5114434", "ggsans-BoldItalic", "5d225b25d66b30563a00f395476ed701130d3f749620a63531cea09fc537164e"),
    ("b2fdbe507d6ce9ef", "ggsans-ExtraBold", "80ea5f1aabbe41c65a0352b56d2be8c409d44b8ab475a14997b7d9986de0029b"),
    ("03dcf979852e8b8e", "ggsans-ExtraBoldItalic", "67165f276046f293a75296f6193cf19607ea65e52988babf95b77f4a7fa2f099"),
];

/// A web font file is about 100 KB; anything much bigger isn't one.
const MOST_BYTES: usize = 1 << 20;

fn path(root: &Path, name: &str) -> PathBuf {
    root.join(FOLDER).join(format!("{name}.ttf"))
}

/// The TrueType files, once every style is downloaded.
pub fn files(root: &Path) -> Option<Vec<PathBuf>> {
    let paths: Vec<PathBuf> = STYLES.iter().map(|(_, name, _)| path(root, name)).collect();
    paths.iter().all(|path| path.is_file()).then_some(paths)
}

/// Downloads the styles not yet in `root`'s font folder. Against the test
/// server it does nothing: the fake has no fonts, and tests draw Yapfast's own.
pub async fn download(root: &Path) -> Result<()> {
    if crate::client::test_server().is_some() {
        bail!("no fonts from the test server");
    }
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let http = reqwest::Client::builder()
        .user_agent(crate::client::user_agent())
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .context("could not set up HTTPS")?;
    let folder = root.join(FOLDER);
    tokio::fs::create_dir_all(&folder)
        .await
        .context("could not make the font folder")?;
    for (asset, name, sha256) in STYLES {
        let target = path(root, name);
        if target.is_file() {
            continue;
        }
        let url = format!("https://discord.com/assets/{asset}.woff2");
        let response = http.get(&url).send().await?.error_for_status()?;
        let woff2 = response.bytes().await?;
        ensure!(woff2.len() <= MOST_BYTES, "{name} is too big for a font");
        ensure!(
            hex(&Sha256::digest(&woff2)) == sha256,
            "{name} isn't the font Yapfast knows"
        );
        let ttf = wuff::decompress_woff2(&woff2)
            .map_err(|error| anyhow::anyhow!("could not unpack {name}: {error:?}"))?;
        // Written beside the target and renamed over it, so a file that's
        // there is whole.
        let partial = folder.join(format!("{name}.ttf.part"));
        tokio::fs::write(&partial, &ttf).await?;
        tokio::fs::rename(&partial, &target).await?;
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styles_are_whole() {
        for (asset, name, sha256) in STYLES {
            assert_eq!(asset.len(), 16, "{name}");
            assert_eq!(sha256.len(), 64, "{name}");
            assert!(sha256.bytes().all(|byte| byte.is_ascii_hexdigit()), "{name}");
        }
    }

    #[test]
    fn files_need_every_style() {
        let root = tempfile::tempdir().unwrap();
        assert!(files(root.path()).is_none());
        std::fs::create_dir_all(root.path().join(FOLDER)).unwrap();
        for (_, name, _) in &STYLES[1..] {
            std::fs::write(path(root.path(), name), b"").unwrap();
        }
        assert!(files(root.path()).is_none());
        std::fs::write(path(root.path(), STYLES[0].1), b"").unwrap();
        assert_eq!(files(root.path()).unwrap().len(), STYLES.len());
    }

    #[test]
    fn hex_is_lower_case() {
        assert_eq!(hex(&[0x0a, 0xff]), "0aff");
    }
}
