use clap::Parser;
use kaf_cli::batch::BatchConfig;
use kaf_cli::cli::Cli;

#[test]
fn cli_parses_concurrency_default() {
    let cli = Cli::try_parse_from(["kaf-cli"]).unwrap();
    assert_eq!(cli.concurrency, 4);
}

#[test]
fn cli_parses_concurrency_explicit() {
    let cli = Cli::try_parse_from(["kaf-cli", "--concurrency", "12"]).unwrap();
    assert_eq!(cli.concurrency, 12);
}

#[test]
fn batch_config_reflects_cli_concurrency() {
    let cli = Cli::try_parse_from(["kaf-cli", "--concurrency", "8"]).unwrap();

    let config = BatchConfig {
        output_dir: cli.output_dir.clone(),
        continue_on_error: cli.continue_on_error,
        max_errors: cli.max_errors,
        dry_run: cli.dry_run,
        show_chapters: cli.show_chapters,
        concurrency: cli.concurrency as usize,
    };

    assert_eq!(config.concurrency, 8);
}
