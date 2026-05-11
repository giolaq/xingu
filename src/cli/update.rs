use anyhow::Result;

pub fn run() -> Result<()> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("giolaq")
        .repo_name("xingu")
        .bin_name("xingu")
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()?
        .update()?;
    println!("Updated to {}", status.version());
    Ok(())
}
