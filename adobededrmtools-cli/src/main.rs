mod requests;

use adobededrmtools::{Acsm, AdobeAccount};
use adobededrmtools_crypto::init_rand as inner_init_rand;
use anyhow::Context;
use clap::Parser;

use requests::ResourceDownloader;

fn init_logger() {
    env_logger::init();
}

fn init_rand() {
    let mut seed = [0; 32];
    getrandom::fill(&mut seed).expect("could not fill bytes");
    inner_init_rand(seed);
}

#[derive(clap::Parser)]
#[command(version, about, long_about = None, name = "adobededrmtools")]
struct Cli {
    #[arg(long, help = "Path to .acsm file")]
    acsm: String,

    #[arg(
        long,
        default_value = "account.json",
        help = "Path to JSON account file"
    )]
    account: String,

    #[arg(
        long,
        default_value = ".",
        help = "Path to directory to write the output resources to"
    )]
    out: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger();
    init_rand();

    let Cli {
        acsm,
        account: account_path,
        out: out_directory,
    } = Cli::parse();

    let out_directory = std::path::Path::new(&out_directory);
    if !out_directory.is_dir() {
        return Err(anyhow::anyhow!(
            "the provided output directory path does not lead to a directory"
        ));
    }

    let write_out_directory = |filename: &str, data: &[u8]| -> anyhow::Result<()> {
        let path = out_directory.join(filename);
        println!("Writing resource to file: {:?}", path);
        std::fs::write(path, data).context("could not write resource to file")?;
        Ok(())
    };

    let http_client = requests::ReqwestHttpClient;
    let resource_downloader = requests::ReqwestResourceDownloader;

    // Load existing account or create a new one.
    let account = if let Ok(account_file) = std::fs::File::open(&account_path) {
        let account: AdobeAccount = serde_json::from_reader(account_file)?;
        println!(
            "Using account loaded from file: {}",
            account.user_credentials.user
        );
        account
    } else {
        println!("No stored account was found. Creating a new Adobe account..");

        let account = adobededrmtools::create_adobe_account(&http_client, Default::default())
            .await
            .context("could not create adobe account")?;

        serde_json::to_writer_pretty(std::fs::File::create(&account_path)?, &account)?;
        println!("Created account info was stored to {}", account_path);
        account
    };

    // Load .acsm file.
    let acsm = Acsm::from_file(&acsm).context("could not read acsm file")?;

    // Fulfill ACSM.
    println!("Fulfilling ACSM..");
    let resources = adobededrmtools::fulfill_acsm(&http_client, &acsm, &account)
        .await
        .context("failed to fulfill acsm")?;

    log::debug!("resources: {:?}", resources);
    println!("Fulfill returned {} resource", resources.len());

    for (i, resource) in resources.into_iter().enumerate() {
        let i = i + 1;
        println!("Downloading resource #{}: {:?}", i, resource.download);

        let downloaded = resource_downloader
            .download_resource(&resource.download)
            .await
            .context("failed to download resource")?;

        let encrypted_resource = &downloaded.data;

        println!("Removing DRM from the downloaded resource..");
        let dedrm_result =
            adobededrmtools::dedrm::ResourceType::from_item_type(&resource.item_type)
                .ok_or_else(|| anyhow::anyhow!("unsupported resource type: {}", resource.item_type))
                .and_then(|resource_type| {
                    let dedrm_resource_result = adobededrmtools::dedrm::dedrm_resource(
                        resource_type,
                        &resource.encrypted_key.encrypted_key,
                        &account.user_credentials.private_license_key,
                        encrypted_resource,
                    )
                    .context("could not dedrm resource");

                    dedrm_resource_result.map(|x| (resource_type, x))
                });

        match dedrm_result {
            Ok((resource_type, decrypted_resource)) => {
                let ext = resource_type.file_extension();
                write_out_directory(&format!("resource_{}.{}", i, ext), &decrypted_resource)?;
            }
            Err(err) => {
                println!("Could not remove DRM from the resource: {:?}", err);
                println!("Storing raw resource.");

                write_out_directory(&format!("resource_{}.raw", i), &encrypted_resource)?;
            }
        }
    }

    println!("Done!");
    Ok(())
}
