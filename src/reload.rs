mod seq;
use super::colors::colordict::ColorDict;
use home::home_dir;
use nix::{sys::signal::{kill, Signal::{SIGKILL, SIGUSR1}}, unistd::Pid};
use notify_rust::{Notification, Urgency::Normal};
use procfs::process::all_processes;
use seq::seq;
use std::{fs::{read_dir, write}, path::Path, process::{Command, Stdio}, thread::sleep, time::Duration};
use tree_magic_mini::from_filepath;
use wall::xlib::{ImageFormat, Xlib};

pub fn reload(dict: ColorDict, skip: String, vte: bool, feh: bool, setting: String) {
	if skip == "a" {
		return;
	}
	let mut proc = String::new();

	for prc in all_processes().unwrap() {
		match &*prc.stat.comm {
			"dunst" => proc.push('d'),
			"polybar" => proc.push('p'),
			"i3" => proc.push('i'),
			"sway" => proc.push('s'),
			_ => (),
		}
	}

	if skip.is_empty() {
		reload_checked(dict, proc, vte, feh, setting)
	} else {
		reload_checked_skips(dict, skip, proc, vte, feh, setting)
	}
}

fn reload_checked(dict: ColorDict, proc: String, vte: bool, usefeh: bool, setting: String) {
	wallpaper(&dict.wallpaper, usefeh, setting);
	pts(dict, vte, true);
	xrdb();
	if proc.contains('p') {
		polybar();
	}
	if proc.contains('d') {
		dunst();
	}
	if proc.contains('i') {
		i3();
	}
	if proc.contains('s') {
		sway();
	}
}

fn reload_checked_skips(dict: ColorDict, skip: String, proc: String, vte: bool, usefeh: bool, setting: String) {
	if !skip.contains('w') {
		wallpaper(&dict.wallpaper, usefeh, setting);
	}
	if !skip.contains('t') {
		pts(dict, vte, !skip.contains('e'));
	}
	if !skip.contains('x') {
		xrdb();
	}
	if proc.contains('p') && !skip.contains('p') {
		polybar();
	}
	if proc.contains('d') && !skip.contains('d') {
		dunst();
	}
	if proc.contains('i') && !skip.contains('i') {
		i3();
	}
	if proc.contains('s') && !skip.contains('s') {
		sway();
	}
}

fn dunst() {
	for prc in all_processes().unwrap() {
		if prc.stat.comm == "dunst" || prc.stat.comm == "/usr/bin/dunst" {
			kill(Pid::from_raw(prc.stat.pid), SIGKILL).expect("SIGTERM failed")
		}
	}
	sleep(Duration::from_millis(1));

	Notification::new()
		.summary("wal")
		.body("Reloaded wal configurations!")
		.urgency(Normal)
		.show()
		.unwrap();
}

fn pts(dict: ColorDict, vte: bool, writeseq: bool) {
	let seq = seq(dict, vte);
	for dir in read_dir("/dev/pts/").unwrap() {
		let file = dir.unwrap().path().display().to_string();
		if !file.contains("ptmx") {
			write(file, &seq).expect("write to /dev/pts failed.");
		}
	}
	if writeseq {
		write(format!("{}/.cache/wal/seq", home_dir().unwrap().display().to_string()), seq).expect("write failed");
	}
}

fn polybar() {
	for prc in all_processes().unwrap() {
		if prc.stat.comm == "polybar" {
			kill(Pid::from_raw(prc.stat.pid), SIGUSR1).expect("SIGUSR1 failed")
		}
	}
}

fn xrdb() {
	droppedcmd(&["xrdb", "-merge", &format!("{}/.cache/wal/colors.Xresources", home_dir().unwrap().display().to_string())]);
}

fn i3() {
	droppedcmd(&["i3-msg", "reload"]);
}

fn sway() {
	droppedcmd(&["swaymsg", "reload"]);
}

fn wallpaper(path: &str, usefeh: bool, setting: String) {
	if usefeh {
		droppedcmd(&["feh", "--no-fehbg", &format!("--bg-{}", validate_setting(setting)), path]);
	} else {
		let path = Path::new(&path);
		Xlib::set(&Xlib::new().unwrap(), path, mime2format(path)).unwrap();
	}
}

fn droppedcmd(command: &[&str]) {
	let _ = Command::new(command[0])
		.args(&command[1..])
		.stdin(Stdio::null())
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.spawn();
}

fn validate_setting(setting: String) -> String {
	if !is_setting(&setting) {
		println!("Setting invalid, using default");
		"fill".to_string()
	} else {
		setting
	}
}

fn mime2format(path: &Path) -> Option<ImageFormat> {
	match from_filepath(path).unwrap() {
		"image/jpeg" => Some(ImageFormat::Jpeg),
		"image/png" => Some(ImageFormat::Png),
		"image/avif" => Some(ImageFormat::Avif),
		"image/bmp" => Some(ImageFormat::Bmp),
		"image/webp" => Some(ImageFormat::WebP),
		"image/tiff" => Some(ImageFormat::Tiff),
		_ => None,
	}
}

fn is_setting(setting: &str) -> bool {
	matches!(setting, "center" | "fill" | "scale" | "tile")
}
