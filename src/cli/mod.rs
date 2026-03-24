pub mod apks;
pub mod apps;
pub mod availability;
pub mod details;
pub mod edits;
pub mod exec;
pub mod helpers;
pub mod images;
pub mod listings;
pub mod reports;
pub mod targeting;
pub mod videos;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Manage authentication
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    /// List and inspect apps
    Apps {
        #[command(subcommand)]
        command: apps::AppsCommands,
    },
    /// Manage edits (draft app versions)
    Edits {
        #[command(subcommand)]
        command: edits::EditsCommands,
    },
    /// Manage APK files
    Apks {
        #[command(subcommand)]
        command: apks::ApksCommands,
    },
    /// Manage store listings per locale
    Listings {
        #[command(subcommand)]
        command: listings::ListingsCommands,
    },
    /// Manage app details
    Details {
        #[command(subcommand)]
        command: details::DetailsCommands,
    },
    /// Manage screenshots and icons
    Images {
        #[command(subcommand)]
        command: images::ImagesCommands,
    },
    /// Upload videos
    Videos {
        #[command(subcommand)]
        command: videos::VideosCommands,
    },
    /// Manage app availability and scheduling
    Availability {
        #[command(subcommand)]
        command: availability::AvailabilityCommands,
    },
    /// Manage APK device targeting
    Targeting {
        #[command(subcommand)]
        command: targeting::TargetingCommands,
    },
    /// Download sales, earnings, and subscription reports
    Reports {
        #[command(subcommand)]
        command: reports::ReportsCommands,
    },
    /// Helper workflows (compound commands)
    #[command(name = "+publish")]
    Publish(helpers::PublishArgs),
    /// Get app status summary
    #[command(name = "+status")]
    Status(helpers::StatusArgs),
    /// Update listing fields directly
    #[command(name = "+update-listing")]
    UpdateListing(helpers::UpdateListingArgs),
}

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Configure API credentials (client ID and secret)
    Setup,
    /// Acquire a fresh OAuth token
    Login,
    /// Print the current access token
    Token,
}
