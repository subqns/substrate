// This file is part of Substrate.

// Copyright (C) 2018-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Implementation of the `echo` subcommand
use crate::error;
use structopt::StructOpt;

/// The `echo` command
#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "echo", about = "Echo back supplied args.")]
pub struct EchoCmd {
	/// No new line should be added
	#[structopt(short = "n")]
	no_newline: bool,

	message: Vec<String>,
}

impl EchoCmd {
	/// Run the command
	pub fn run(&self) -> error::Result<()> {
		let line = self.message.join(" ");
		if self.no_newline {
			print!("{}", line);
		} else {
			println!("{}", line);
		}
		Ok(())
	}
}
