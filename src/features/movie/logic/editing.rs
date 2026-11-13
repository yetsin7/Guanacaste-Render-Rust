//! Editing operations on an [`Animation`]: add, remove and move keys while keeping every list sorted by time.

use crate::features::movie::models::{
    Animation, CameraKey, CameraPose, Channel, ValueKey, FPS_RANGE, MAX_DURATION, SPEED_RANGE,
};

/// Keys closer than this (in seconds) are considered the same instant.
pub const SAME_TIME: f64 = 1e-3;

fn clamp_time(time: f64) -> f64 {
    if time.is_finite() {
        time.clamp(0.0, MAX_DURATION)
    } else {
        0.0
    }
}

impl Animation {
    /// Adds a camera key with `pose` at `time`; a key already at that instant gets the new pose instead.
    /// Returns the index of the key.
    pub fn add_camera_key(&mut self, time: f64, pose: CameraPose) -> usize {
        let time = clamp_time(time);
        let keys = &mut self.camera.keys;
        let index = keys.partition_point(|key| key.time < time - SAME_TIME);
        if keys
            .get(index)
            .is_some_and(|key| key.time - time < SAME_TIME)
        {
            keys[index].pose = pose.sanitized();
            return index;
        }
        keys.insert(
            index,
            CameraKey {
                time,
                pose: pose.sanitized(),
                ..CameraKey::default()
            },
        );
        index
    }

    /// Records the live camera as a key at `time` (see [`Animation::add_camera_key`]).
    pub fn record_camera(&mut self, time: f64, camera: &CameraPose) -> usize {
        self.add_camera_key(time, *camera)
    }

    /// Removes a camera key; returns whether it existed.
    pub fn remove_camera_key(&mut self, index: usize) -> bool {
        if index < self.camera.keys.len() {
            self.camera.keys.remove(index);
            true
        } else {
            false
        }
    }

    /// Moves a camera key to `time` and returns its new index (the list stays sorted).
    pub fn move_camera_key(&mut self, index: usize, time: f64) -> usize {
        let Some(mut key) =
            (index < self.camera.keys.len()).then(|| self.camera.keys.remove(index))
        else {
            return index;
        };
        key.time = clamp_time(time);
        let at = self
            .camera
            .keys
            .partition_point(|other| other.time <= key.time);
        self.camera.keys.insert(at, key);
        at
    }

    /// Adds a channel called `name` (made unique with a numeric suffix) and returns its index.
    pub fn add_channel(&mut self, name: &str) -> usize {
        let base = if name.trim().is_empty() {
            "canal"
        } else {
            name.trim()
        };
        let mut unique = base.to_owned();
        let mut number = 2;
        while self.channels.iter().any(|channel| channel.name == unique) {
            unique = format!("{base} {number}");
            number += 1;
        }
        self.channels.push(Channel {
            name: unique,
            keys: Vec::new(),
        });
        self.channels.len() - 1
    }

    /// Removes a channel; returns whether it existed.
    pub fn remove_channel(&mut self, channel: usize) -> bool {
        if channel < self.channels.len() {
            self.channels.remove(channel);
            true
        } else {
            false
        }
    }

    /// Adds a key to a channel (replacing the value of a key at the same instant); returns its index.
    pub fn add_value_key(&mut self, channel: usize, time: f64, value: f64) -> Option<usize> {
        let time = clamp_time(time);
        let value = if value.is_finite() { value } else { 0.0 };
        let keys = &mut self.channels.get_mut(channel)?.keys;
        let index = keys.partition_point(|key| key.time < time - SAME_TIME);
        if keys
            .get(index)
            .is_some_and(|key| key.time - time < SAME_TIME)
        {
            keys[index].value = value;
            return Some(index);
        }
        keys.insert(
            index,
            ValueKey {
                time,
                value,
                ..ValueKey::default()
            },
        );
        Some(index)
    }

    /// Removes a key of a channel; returns whether it existed.
    pub fn remove_value_key(&mut self, channel: usize, key: usize) -> bool {
        match self.channels.get_mut(channel) {
            Some(channel) if key < channel.keys.len() => {
                channel.keys.remove(key);
                true
            }
            _ => false,
        }
    }

    /// Moves a key of a channel to `time` and returns its new index.
    pub fn move_value_key(&mut self, channel: usize, key: usize, time: f64) -> usize {
        let Some(keys) = self.channels.get_mut(channel).map(|c| &mut c.keys) else {
            return key;
        };
        if key >= keys.len() {
            return key;
        }
        let mut moved = keys.remove(key);
        moved.time = clamp_time(time);
        let at = keys.partition_point(|other| other.time <= moved.time);
        keys.insert(at, moved);
        at
    }

    /// Repairs data that came from a file: finite values, sorted keys, frame rate, speed and duration in range.
    pub fn normalize(&mut self) {
        self.fps = self.fps.clamp(FPS_RANGE.0, FPS_RANGE.1);
        self.speed = if self.speed.is_finite() {
            self.speed.clamp(SPEED_RANGE.0, SPEED_RANGE.1)
        } else {
            1.0
        };
        self.duration = if self.duration.is_finite() {
            self.duration.clamp(0.0, MAX_DURATION)
        } else {
            Animation::default().duration
        };
        for key in &mut self.camera.keys {
            key.time = clamp_time(key.time);
            key.pause = clamp_time(key.pause);
            key.pose = key.pose.sanitized();
        }
        self.camera.keys.sort_by(|a, b| a.time.total_cmp(&b.time));
        for channel in &mut self.channels {
            for key in &mut channel.keys {
                key.time = clamp_time(key.time);
                key.value = if key.value.is_finite() {
                    key.value
                } else {
                    0.0
                };
            }
            channel.keys.sort_by(|a, b| a.time.total_cmp(&b.time));
        }
    }
}
