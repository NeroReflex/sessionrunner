/*
    login-ng A greeter written in rust that also supports autologin with systemd-homed
    Copyright (C) 2024-2025  Denis Benato

    This program is free software; you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation; either version 2 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along
    with this program; if not, write to the Free Software Foundation, Inc.,
    51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
*/

use argh::FromArgs;
use sessionrunner::dbus::SessionManagerDBusProxy;
use zbus::Connection;

#[derive(FromArgs, PartialEq, Debug)]
/// Command line tool for managing sessionrunner
struct Args {
    #[argh(option, short = 't')]
    /// the target to be started/stopped/restarted or the subtree to be evaluated
    target: Option<String>,

    #[argh(subcommand)]
    command: Command,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
/// Subcommands for managing sessionrunner
enum Command {
    Inspect(InspectCommand),
    Start(StartCommand),
    Stop(StopCommand),
    Restart(RestartCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Inspect a target and its dependencies
#[argh(subcommand, name = "inspect")]
struct InspectCommand {}

#[derive(FromArgs, PartialEq, Debug)]
/// Start a target from within sessionrunner
#[argh(subcommand, name = "start")]
struct StartCommand {}

#[derive(FromArgs, PartialEq, Debug)]
/// Stop a target from within sessionrunner
#[argh(subcommand, name = "stop")]
struct StopCommand {}

#[derive(FromArgs, PartialEq, Debug)]
/// Restart a target from within sessionrunner
#[argh(subcommand, name = "restart")]
struct RestartCommand {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::session().await?;
    let proxy = SessionManagerDBusProxy::new(&connection).await?;

    let args: Args = argh::from_env();

    let target = match &args.target {
        Some(t) => t.clone(),
        None => String::from("default.service"),
    };

    match &args.command {
        Command::Stop(_stop_command) => {
            proxy.stop(target).await.unwrap();
        }
        Command::Restart(_restart_command) => {
            proxy.restart(target).await.unwrap();
        }
        Command::Start(_start_command) => {
            proxy.start(target).await.unwrap();
        }
        Command::Inspect(_inspect_command) => {
            let (status, result) = proxy.inspect(target).await.unwrap();
            if status == 0 {
                println!("{result}")
            } else {
                panic!("inspect errorer with {status}: {result}")
            }
        }
    }

    Ok(())
}
