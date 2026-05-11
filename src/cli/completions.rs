use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell};

pub fn run<C: CommandFactory>(shell: Shell) -> Result<()> {
    let mut cmd = C::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut std::io::stdout());
    Ok(())
}
