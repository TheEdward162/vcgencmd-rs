use clap::{App, Arg, ArgMatches};

use videocore_gencmd::prelude::*;

fn parse_cli() -> ArgMatches<'static> {
	App::new(env!("CARGO_PKG_NAME"))
		.version(env!("CARGO_PKG_VERSION"))
		.arg(
			Arg::with_name("raw")
				.short("r")
				.long("raw")
				.takes_value(false)
				.help("Do not attempt to recognize the command nor parse the response (errors are always parsed)")
		)
		.arg(
			Arg::with_name("verbosity")
				.short("v")
				.long("verbosity")
				.takes_value(true)
				.default_value("Off")
				.possible_values(&["Off", "Error", "Warn", "Info", "Debug", "Trace"])
				.help("Level of verbosity")
		)
		.arg(
			Arg::with_name("command").required(true).index(1).multiple(true)
		)
		.get_matches()
}

fn setup_logger(level: log::Level) {
	edwardium_logger::Logger::new(
		edwardium_logger::targets::stderr::StderrTarget::new(level, Default::default()),
		std::time::Instant::now()
	)
	.init_boxed()
	.expect("Could not initialize logger");
}

fn main() -> anyhow::Result<()> {
	let matches = parse_cli();

	if let Some(level) = match matches.value_of("verbosity").unwrap() {
		"Off" => None,
		"Error" => Some(log::Level::Error),
		"Warn" => Some(log::Level::Warn),
		"Info" => Some(log::Level::Info),
		"Debug" => Some(log::Level::Debug),
		"Trace" => Some(log::Level::Trace),
		_ => unreachable!()
	} {
		setup_logger(level);
		log::debug!("{:?}", matches);
	}

	let raw = matches.is_present("raw");
	let mut command = matches.values_of("command").unwrap();

	let mut gencmd = GencmdUnique::new()?;

	if raw {
		let first_command = command.next().unwrap().to_string();
		let command = command.fold(first_command, |mut acc, curr| {
			acc.push(' ');
			acc.push_str(curr);
			acc
		});

		let response = gencmd.send_cmd_raw(&command)?;
		println!("{}", response);

		return Ok(())
	}

	match command.next().unwrap() {
		"commands" => gencmd.send_cmd::<CmdCommands>()?.into_iter().for_each(|command| println!("{}", command)),
		"measure_temp" => {
			let temp = gencmd.send_cmd::<CmdMeasureTemp>()?;
			println!("{}", temp);
		}
		"get_throttled" => {
			let throttled = gencmd.send_cmd::<CmdGetThrottled>()?;
			println!("0x{:X}", u32::from(throttled));
		}
		"measure_clock" => match command.next() {
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
