use lazy_static::lazy_static;
use log::info;
use std::fs::{create_dir_all, rename};
use xtask_wasm::{anyhow::Result, clap};

#[derive(clap::Parser)]
struct Opt {
    #[clap(long = "log", default_value = "Info")]
    log_level: log::LevelFilter,
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(clap::Parser)]
enum Command {
    Dist(Build),
}

#[derive(clap::Parser)]
struct Build {
    #[clap(flatten)]
    base: xtask_wasm::Dist,
    /// The package to build
    #[clap(index = 1)]
    package: Option<String>,

    #[clap(long)]
    dir_name: Option<String>,
}

fn main() -> Result<()> {
    let opt: Opt = clap::Parser::parse();

    env_logger::builder()
        .filter(Some("bevy_wasm_pack"), opt.log_level)
        .filter(Some("xtask"), opt.log_level)
        .init();

    match opt.cmd {
        Command::Dist(mut arg) => {
            let package_name = arg.package.as_ref().unwrap_or_else(|| {
                &cargo_data()
                    .root_package()
                    .expect(
                        // todo: maybe we should just build all ultra packages then?
                        "No root crate, please provide rom crate name or run from rom subdirectory",
                    )
                    .name
            });
            let workspace_root = &cargo_data().workspace_root;
            let dist_root = format!("{workspace_root}/dist");

            info!("Generating package: {package_name}...");

            arg.base.release = true;

            let dir_name = arg.dir_name.as_ref().unwrap_or(package_name);
            let dist_dir = format!("{dist_root}/{dir_name}");

            let dist_result = arg.base.run(package_name)?;

            xtask_wasm::WasmOpt::level(3)
                .shrink(3)
                .optimize(&dist_result.wasm)?;

            let size = std::fs::metadata(&dist_result.wasm)?.len();
            info!("File size: {}", bytesize::ByteSize(size));

            info!("Creating dist dir: {dist_dir}");
            create_dir_all(&dist_dir)?;

            // info!("File {}", dist_result.js);
            if let Err(err) = rename(&dist_result.wasm, format!("{dist_dir}/app.wasm")) {
                eprintln!("Error renaming: {err:?}");
            }
            if let Err(err) = rename(&dist_result.js, format!("{dist_dir}/app.js")) {
                eprintln!("Error renaming: {err:?}");
            }

            // index as well?
        }
    }

    Ok(())
}

pub fn cargo_data() -> &'static cargo_metadata::Metadata {
    lazy_static! {
        static ref METADATA: cargo_metadata::Metadata = cargo_metadata::MetadataCommand::new()
            .exec()
            .expect("cannot get crate's metadata");
    }

    &METADATA
}
