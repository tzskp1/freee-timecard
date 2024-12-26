use anyhow::Result;
use clap::{Parser, Subcommand};
use headless_chrome::{Browser, Element, LaunchOptions, Tab};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[clap(long, short, action)]
    non_headless: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    BreakStart,
    BreakEnd,
    ClockIn,
    ClockOut,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct Settings {
    email: String,
    password: String,
}

fn get_button<'a>(
    mut buttons: impl Iterator<Item = &'a Element<'a>>,
    text: &str,
) -> Option<&'a Element<'a>> {
    buttons.find(|e| {
        e.get_inner_text()
            .map(|t| t.contains(text))
            .unwrap_or(false)
    })
}

fn goto_login_page(tab: &Tab) -> Result<()> {
    tab.navigate_to("https://www.freee.co.jp/hr/")?
        .wait_until_navigated()?;
    let goto_login = tab.find_elements(
        "#gatsby-focus-wrapper > header > div.g-header_inner > div > div.g-headerBtn > a",
    )?;
    let goto_login =
        get_button(goto_login.iter(), "ログインする").expect("Failed to find login button");
    goto_login.click()?;
    Ok(())
}

fn login(tab: &Tab, cfg: &Settings) -> Result<()> {
    let inputs = tab.wait_for_elements("input")?;
    inputs[0].click()?;
    tab.send_character(&cfg.email)?;
    inputs[1].click()?;
    tab.send_character(&cfg.password)?.press_key("Enter")?;
    Ok(())
}

fn click_target_button(tab: &Tab, cmd: Commands) -> Result<()> {
    let buttons = tab.wait_for_elements("#global-navigation-body-block button")?;
    let button = match cmd {
        Commands::BreakStart => get_button(buttons.iter(), "休憩開始"),
        Commands::BreakEnd => get_button(buttons.iter(), "休憩終了"),
        Commands::ClockIn => get_button(buttons.iter(), "出勤"),
        Commands::ClockOut => get_button(buttons.iter(), "退勤"),
    };
    button.expect("Failed to find target button").click()?;
    tab.wait_until_navigated()?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg: Settings = confy::load("freee-timecard", None)?;
    let options = LaunchOptions::default_builder()
        .window_size(Some((2048, 2048)))
        .headless(!cli.non_headless)
        .build()
        .expect("Couldn't find appropriate Chrome binary.");
    let browser = Browser::new(options)?;
    let tab = browser.new_tab()?;
    goto_login_page(&tab)?;
    login(&tab, &cfg)?;
    click_target_button(&tab, cli.command)?;
    Ok(())
}
