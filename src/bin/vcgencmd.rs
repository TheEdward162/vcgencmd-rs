use clap::{Parser, ValueEnum};

use videocore_gencmd::prelude::*;

#[derive(Debug, Clone, ValueEnum)]
enum Verbosity {
	Off,
	Error,
	Warn,
	Info,
	Debug,
	Trace,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
	/// Do not attempt to recognize the command nor parse the response (errors are always parsed)
	#[arg(short, long)]
	pub raw: bool,
	#[arg(short, long, default_value = "off")]
	pub verbosity: Verbosity,
	#[arg(required = true)]
	pub command: Vec<String>,
}

fn setup_logger(level: log::Level) {
	edwardium_logger::Logger::new(
		edwardium_logger::targets::stderr::StderrTarget::new(level, Default::default()),
		std::time::Instant::now(),
	)
	.init_boxed()
	.expect("Could not initialize logger");
}

fn main() -> anyhow::Result<()> {
	let cli = Cli::parse();

	if let Some(level) = match cli.verbosity {
		Verbosity::Off => None,
		Verbosity::Error => Some(log::Level::Error),
		Verbosity::Warn => Some(log::Level::Warn),
		Verbosity::Info => Some(log::Level::Info),
		Verbosity::Debug => Some(log::Level::Debug),
		Verbosity::Trace => Some(log::Level::Trace),
	} {
		setup_logger(level);
		log::debug!("{:?}", cli);
	}

	let mut command = cli.command.into_iter();
	let mut gencmd = GencmdUnique::new()?;
	if cli.raw {
		let first_command = command.next().unwrap();
		let command = command.fold(first_command, |mut acc, curr| {
			acc.push(' ');
			acc.push_str(&curr);
			acc
		});

		let response = gencmd.send_cmd_raw(&command)?;
		println!("{}", response);

		return Ok(());
	}

	match command.next().unwrap().as_str() {
		"commands" => gencmd.send_cmd::<CmdCommands>()?.into_iter().for_each(|command| println!("{}", command)),
		"measure_temp" => {
			let temp = gencmd.send_cmd::<CmdMeasureTemp>()?;
			println!("{}", temp);
		}
		"get_throttled" => {
			let throttled = gencmd.send_cmd::<CmdGetThrottled>()?;
			println!("0x{:X}", u32::from(throttled));
		}
		"measure_clock" => match command.next().as_deref() {
			Some("arm") => {
				let freq = gencmd.send_cmd::<CmdMeasureClockArm>()?;
				println!("{}", freq);
			}
			_ => anyhow::bail!("unrecognized arguments to `measure_clock`, try again with `--raw` or add implementation"),
		}
		_ => anyhow::bail!("unrecognized command, try again with `--raw` or add implementation")
	}

	Ok(())
}
