use tokio::{select, sync::mpsc};

use crate::send;

pub fn requires_plugin(
    mut battery_level: mpsc::Receiver<u16>,
    mut plugged_in: mpsc::Receiver<bool>,
    mut geofence: mpsc::Receiver<String>,
    mut reminder: mpsc::Receiver<bool>,
) -> mpsc::Receiver<bool> {
    let (tx, rx) = mpsc::channel(10);
    tokio::spawn(async move {
        let mut the_battery_level: Option<u16> = None;
        let mut the_plugged_in: Option<bool> = None;
        let mut the_geofence: Option<String> = None;
        let mut the_reminder: Option<bool> = None;

        loop {
            select! {
                Some(battery_level) = battery_level.recv() => { the_battery_level = Some(battery_level)},
                Some(plugged_in) = plugged_in.recv() => { the_plugged_in = Some(plugged_in)},
                Some(geofence) = geofence.recv() => { the_geofence = Some(geofence)},
                Some(reminder) = reminder.recv() => { the_reminder = Some(reminder)},
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
                    send(&tx, true).await;
                }
                (_, _, _, _) => {
                    send(&tx, false).await;
                }
            };
        }
    });

    rx
}

pub fn is_insecure(
    mut is_user_present: mpsc::Receiver<bool>,
    mut locked: mpsc::Receiver<bool>,
) -> mpsc::Receiver<bool> {
    let (tx, rx) = mpsc::channel(10);
    tokio::spawn(async move {
        let mut the_is_user_present: Option<bool> = None;
        let mut the_locked: Option<bool> = None;

        loop {
            select! {
                Some(is_user_present) = is_user_present.recv() => { the_is_user_present = Some(is_user_present)},
                Some(locked) = locked.recv() => { the_locked = Some(locked)},
                else => { break; }
            }

            match (the_is_user_present, the_locked) {
                (None, _) => {}
                (_, None) => {}
                (Some(false), Some(false)) => {
                    send(&tx, false).await;
                }
                (_, _) => {
                    send(&tx, false).await;
                }
            };
        }
    });

    rx
}