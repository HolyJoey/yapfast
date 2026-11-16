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

/// The fonts of Discord's display name styles whose licences don't allow
/// passing them on, as the web client loads them for Neo Castel (font id 7,
/// family "Medieval") and Sinistre (10, "Vampyre"). The others are free
/// fonts that ship with Yapfast; see `ui/fonts/styles`.
const NAME_STYLES: [(&str, &str, &str); 2] = [
    ("52b541f86401a5b6", "Medieval-Regular", "2d38598492e7312d3ccaa2e98642a04d1deb7a38089ab201f8d6852b6907de40"),
    ("8f20cb550d739cea", "Vampyre-Regular", "c529d168e2c030d999db38cf3b188a33e76b6d1e19a3dbd48b4b047f0b134b5b"),
];

/// Discord's own sounds, as its web client plays them: the file under
/// `discord.com/assets/`, the name Yapfast keeps it under in [`SOUNDS`],
/// and the SHA-256 of the MP3 Discord serves. Like gg sans, they're
/// downloaded, never shipped.
const SOUND_FILES: [(&str, &str, &str); 13] = [
    ("3ed22d14f3c30bc4", "message1", "31ad0482eee7770597b8aa723a80fd041ade0b076679b12293664f1f1777211b"),
    ("b135ff6c8e091b43", "user_join", "d30746caf3e4675ae0d822d51461a9ad24832afa1e20179c3c2fc7b50b911a26"),
    ("7b9a183742515fc2", "user_leave", "9fd71c2d8112c82a7fb316602bb1645bc65f5edfa260110bbaae80090fbe9df0"),
    ("2d3b4ba32c34c862", "mute", "f6194168829b0701e8b40817d5173afed4b3b1e0b5074ab82ca31d97e4cb65c1"),
    ("e74c4a06134a20e4", "unmute", "1572881f90703c1e0cd138fe7486d2e53c0ac5d8509cade32029fb31650b9304"),
    ("529ff198eac567af", "deafen", "dee4468bbafb321b159dcab42f52d1fbfb1d01358437e0a3088c3345979211b8"),
    ("b150f03c89944403", "undeafen", "690b64977594baa41c7978d76259224f67799ba337df2d1045dce970ef82b243"),
    ("abe52a3c92953edb", "stream_started", "b5cb29d5d5cc0e8e22fa014bac4a1c2d601f6890ae6db1c18d4b6310283a3271"),
    ("cbff07b5a83e0321", "stream_ended", "a724e183167405783122b997def6b3e6f53020957031224b27c434d6df8ef75f"),
    ("3d2af2764581bb2e", "stream_user_joined", "3b51208e7559fc5b16a297cd8d882648fa7534adf706dfbd529c39266d132213"),
    ("24ae9dc0184bd570", "stream_user_left", "be2f1374b1ec8137eaa7e11357aafe12a3f3e09c0ed759b5d11043e723992061"),
    ("c2a7111bb44b8da0", "call_ringing", "a2365a04f839099538271d06889147475ceef0845f8cc010425618f5dc412880"),
    ("5146737af413d88e", "call_calling", "c3999dbbbea7fca113d6f396b81d6054dd2dd6df79f442b62dfb74afddd36934"),
];

/// The folder under the data folder that holds Discord's sounds.
pub const SOUNDS: &str = "sounds";

fn sound_path(root: &Path, name: &str) -> PathBuf {
    root.join(SOUNDS).join(format!("{name}.mp3"))
}

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

/// The display name styles' files that are downloaded, each on its own.
pub fn name_files(root: &Path) -> Vec<PathBuf> {
    NAME_STYLES
        .iter()
        .map(|(_, name, _)| path(root, name))
        .filter(|path| path.is_file())
        .collect()
}

/// Whether every font and sound Yapfast downloads is there.
pub fn complete(root: &Path) -> bool {
    files(root).is_some()
        && name_files(root).len() == NAME_STYLES.len()
        && SOUND_FILES.iter().all(|(_, name, _)| sound_path(root, name).is_file())
}

/// Downloads the styles not yet in `root`'s font folder, gg sans's, then
/// the display name styles'. Against the test
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
    for (asset, name, sha256) in STYLES.into_iter().chain(NAME_STYLES) {
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
    // Then Discord's sounds, kept as Discord serves them.
    tokio::fs::create_dir_all(root.join(SOUNDS))
        .await
        .context("could not make the sounds folder")?;
    for (asset, name, sha256) in SOUND_FILES {
        let target = sound_path(root, name);
        if target.is_file() {
            continue;
        }
        let url = format!("https://discord.com/assets/{asset}.mp3");
        let response = http.get(&url).send().await?.error_for_status()?;
        let mp3 = response.bytes().await?;
        ensure!(mp3.len() <= MOST_BYTES, "{name} is too big for a sound");
        ensure!(
            hex(&Sha256::digest(&mp3)) == sha256,
            "{name} isn't the sound Yapfast knows"
        );
        let partial = root.join(SOUNDS).join(format!("{name}.mp3.part"));
        tokio::fs::write(&partial, &mp3).await?;
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
        for (asset, name, sha256) in STYLES.into_iter().chain(NAME_STYLES).chain(SOUND_FILES) {
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
        assert!(!complete(root.path()), "without the name styles");
        std::fs::write(path(root.path(), NAME_STYLES[1].1), b"").unwrap();
        assert_eq!(name_files(root.path()).len(), 1, "each on its own");
        std::fs::write(path(root.path(), NAME_STYLES[0].1), b"").unwrap();
        assert!(!complete(root.path()), "without Discord's sounds");
        std::fs::create_dir_all(root.path().join(SOUNDS)).unwrap();
        for (_, name, _) in SOUND_FILES {
            std::fs::write(sound_path(root.path(), name), b"").unwrap();
        }
        assert!(complete(root.path()));
    }

    #[test]
    fn hex_is_lower_case() {
        assert_eq!(hex(&[0x0a, 0xff]), "0aff");
    }
}
