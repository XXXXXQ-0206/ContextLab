#![forbid(unsafe_code)]

//! Executable entry point for the ContextLab CLI staging shell.

use std::process::ExitCode;

use contextlab_adapter_contract::UnavailableApplicationAdapter;
use contextlab_cli::{
    execute, inspect_serialized_local_capability_availability,
    inspect_serialized_replay_state_snapshot, parse_arguments,
};

fn main() -> ExitCode {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();

    match arguments.as_slice() {
        [resource, action, serialized] if resource == "capability" && action == "inspect" => {
            return match inspect_serialized_local_capability_availability(serialized) {
                Ok(inspection) => {
                    print!("{}", inspection.render());
                    ExitCode::from(2)
                }
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::from(64)
                }
            };
        }
        [resource, action, serialized] if resource == "replay" && action == "inspect" => {
            return match inspect_serialized_replay_state_snapshot(serialized) {
                Ok(inspection) => {
                    print!("{}", inspection.render());
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::from(64)
                }
            };
        }
        _ => {}
    }

    match parse_arguments(arguments) {
        Ok(request) => match execute(&UnavailableApplicationAdapter, request) {
            Ok(execution) => {
                println!("{}", execution.message);
                ExitCode::from(2)
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::from(64)
            }
        },
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(64)
        }
    }
}
