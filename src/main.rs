mod twitch;

use std::env;
use std::fs::{File, rename};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io, thread};
use std::env::current_dir;
use std::time::Duration;
use log::{error, info};
use cached::proc_macro::cached;

extern crate simplelog;
extern crate core;

use simplelog::*;
use crate::twitch::ChannelInfo;

const LOG_FILE: &'static str = "stream-archiver.log";
const RECORDING_INDICATOR_DIR: &'static str = ".recording";

fn main() {
    init_logging();
    let archive_dir = get_archive_dir();
    init_recording_indicator_dir(&archive_dir);
    info!("Recording to {:?}", &archive_dir);

    loop {
        let channel_names: Vec<String> = read_channels();

        info!("Checking {} channels", channel_names.len());
        let live_channels: Vec<ChannelInfo> = twitch::get_live_channels(&channel_names);
        info!("Live Channels: {:?}", live_channels.iter().map(|c| c.user_name.as_str()).collect::<Vec<_>>());

        let mut still_recording: Vec<String> = Vec::new();
        for channel in live_channels {
            if is_recording(&archive_dir, &channel.user_login) {
                still_recording.push(channel.user_name);
            } else {
                let dir = archive_dir.clone();
                thread::spawn(move ||
                    {
                        try_record(&dir, channel);
                    }
                );
            }
        }
        info!("Still recording: {:?}", still_recording);

        thread::sleep(Duration::from_secs(30));
    }
}

fn get_archive_dir() -> PathBuf {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        current_dir().unwrap()
    }
}

fn init_recording_indicator_dir(archive_dir: &PathBuf) {
    // A .recording folder is used to keep indicators of channels currently being recorded;
    let path = archive_dir.join(RECORDING_INDICATOR_DIR);
    if path.exists() {
        fs::remove_dir_all(path.clone()).unwrap();
    }
    fs::create_dir(path).unwrap();
}

fn is_recording(archive_dir: &PathBuf, channel_name: &String) -> bool {
    archive_dir.join(RECORDING_INDICATOR_DIR).join(channel_name).exists()
}

#[cached(size = 1, time = 86_400)]
fn read_channels() -> Vec<String> {
    info!("Reading channels.txt");
    let file = File::open("channels.txt").expect("channels.txt missing!");
    let buf = BufReader::new(file);

    let channels: Vec<String> = buf.lines()
        .map(|l| l.expect("Could not parse line"))
        .filter(|c| !c.trim().is_empty())
        .collect();

    if channels.is_empty() {
        error!("Must specify channels in channels.txt");
        panic!();
    }

    info!("Active channels for recording: {:?}", &channels);
    channels
}

fn try_record(archive_dir: &PathBuf, channel: ChannelInfo) {
    let record_command = create_command(archive_dir, &channel);
    File::create(archive_dir.join(RECORDING_INDICATOR_DIR).join(&channel.user_login)).unwrap();

    info!("Starting to record {} with: {}", &channel.user_login, &record_command);
    let rec = Command::new("sh")
        .arg("-c")
        .arg(record_command)
        .output()
        .expect("Could not execute command");

    info!("Recording ended: {}", String::from_utf8(rec.stdout).unwrap());
    fs::remove_file(archive_dir.join(RECORDING_INDICATOR_DIR).join(&channel.user_login)).unwrap();
}

fn create_command(archive_dir: &PathBuf, channel: &ChannelInfo) -> String {
    let login = &channel.user_login;
    let user_name = &channel.user_name;
    let game_name = &channel.game_name;
    let title = &channel.title;
    let time = get_current_time_iso_formatted();
    let filename = format!("{time}_{user_name}_{game_name}_{title}.mkv");
    let target = archive_dir.join(&channel.user_name).join(filename).into_os_string().into_string().unwrap();
    format!("streamlink https://twitch.tv/{login} best --twitch-disable-hosting -o \"{target}\"")
}

fn init_logging() {
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        WriteLogger::new(LevelFilter::Debug, Config::default(), get_log_file().unwrap()),
    ]).unwrap();
}

fn get_log_file() -> Result<File, io::Error> {
    if Path::new(LOG_FILE).exists() {
        fs::create_dir_all("log").expect("Could not create log folder");
        let iso_formatted = get_current_time_iso_formatted() + ".log";
        rename(Path::new(LOG_FILE),
               Path::new("log").join(Path::new(&iso_formatted)))
            .expect("Could not rename log file");
    }
    File::create(LOG_FILE)
}

fn get_current_time_iso_formatted() -> String {
    let ts = time_format::now().unwrap();
    time_format::strftime_utc("%Y-%m-%d_%H-%M-%S", ts).unwrap()
}
