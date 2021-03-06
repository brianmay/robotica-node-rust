//! Filter functions specific to teslamate.
use tokio::{select, sync::broadcast};

use crate::{recv, send_or_log, spawn, Pipe, RxPipe};

fn _requires_plugin(
    mut battery_level: broadcast::Receiver<usize>,
    mut plugged_in: broadcast::Receiver<bool>,
    mut geofence: broadcast::Receiver<String>,
    mut reminder: broadcast::Receiver<bool>,
    output: broadcast::Sender<bool>,
) {
    spawn(async move {
        let mut the_battery_level: Option<usize> = None;
        let mut the_plugged_in: Option<bool> = None;
        let mut the_geofence: Option<String> = None;
        let mut the_reminder: Option<bool> = None;

        loop {
            select! {
                Ok(battery_level) = recv(&mut battery_level) => { the_battery_level = Some(battery_level)},
                Ok(plugged_in) = recv(&mut plugged_in) => { the_plugged_in = Some(plugged_in)},
                Ok(geofence) = recv(&mut geofence) => { the_geofence = Some(geofence)},
                Ok(reminder) = recv(&mut reminder) => { the_reminder = Some(reminder)},
                else => { break; }
            }

            match (
                the_battery_level,
                the_plugged_in,
                the_geofence.as_deref(),
                the_reminder,
            ) {
                (None, _, _, _) => {}
                (_, None, _, _) => {}
                (_, _, None, _) => {}
                (_, _, _, None) => {}
                (Some(level), Some(false), Some("Home"), Some(true)) if level < 75 => {
                    send_or_log(&output, true);
                }
                (_, _, _, _) => {
                    send_or_log(&output, false);
                }
            };
        }
    });
}

fn _is_insecure(
    mut is_user_present: broadcast::Receiver<bool>,
    mut locked: broadcast::Receiver<bool>,
    output: broadcast::Sender<bool>,
) {
    spawn(async move {
        let mut the_is_user_present: Option<bool> = None;
        let mut the_locked: Option<bool> = None;

        loop {
            select! {
                Ok(is_user_present) = recv(&mut is_user_present) => { the_is_user_present = Some(is_user_present)},
                Ok(locked) = recv(&mut locked) => { the_locked = Some(locked)},
                else => { break; }
            }

            match (the_is_user_present, the_locked) {
                (None, _) => {}
                (_, None) => {}
                (Some(false), Some(false)) => {
                    send_or_log(&output, true);
                }
                (_, _) => {
                    send_or_log(&output, false);
                }
            };
        }
    });
}

/// Try to determine if the car requires a plugin.
pub fn requires_plugin(
    battery_level: RxPipe<usize>,
    plugged_in: RxPipe<bool>,
    geofence: RxPipe<String>,
    reminder: RxPipe<bool>,
) -> RxPipe<bool> {
    let output = Pipe::new();
    _requires_plugin(
        battery_level.subscribe(),
        plugged_in.subscribe(),
        geofence.subscribe(),
        reminder.subscribe(),
        output.get_tx(),
    );
    output.to_rx_pipe()
}

/// Try to determine if car is insecure.
pub fn is_insecure(is_user_present: RxPipe<bool>, locked: RxPipe<bool>) -> RxPipe<bool> {
    let output = Pipe::new();
    _is_insecure(
        is_user_present.subscribe(),
        locked.subscribe(),
        output.get_tx(),
    );
    output.to_rx_pipe()
}
