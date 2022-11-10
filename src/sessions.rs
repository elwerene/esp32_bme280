use anyhow::Result;
use serde::Serialize;
use std::{
    fs::File,
    io::{Read, Seek, Write},
    time::{SystemTime, UNIX_EPOCH},
};

const FILE: &str = "sess";
const END_OF_SESSION: u8 = 0xff;

#[derive(Serialize)]
pub struct Session {
    start_at: u64,
    temperatures: Vec<u8>,
}

pub fn init() -> Result<()> {
    let mut file = file()?;

    let pos = file.seek(std::io::SeekFrom::End(0))?;
    if pos > 0 {
        file.write_all(&[END_OF_SESSION])?;
    }

    let now = time()?;
    file.write_all(&now.to_be_bytes())?;
    file.sync_data()?;

    Ok(())
}

pub fn add_temp(temp: u8) -> Result<()> {
    let mut file = file()?;

    file.seek(std::io::SeekFrom::End(0))?;
    file.write_all(&[temp])?;
    file.sync_data()?;

    Ok(())
}

pub fn sessions(remove_old: bool) -> Result<Vec<Session>> {
    let mut sessions = vec![];

    let mut file = file()?;
    file.seek(std::io::SeekFrom::Start(0))?;

    let mut buf = [0u8; 8];
    while file.read_exact(&mut buf).is_ok() {
        let start_at = u64::from_be_bytes(buf);
        let mut temperatures = vec![];
        let mut buf = [0u8];
        while file.read_exact(&mut buf).is_ok() {
            match buf[0] {
                END_OF_SESSION => break,
                temperature => temperatures.push(temperature),
            }
        }
        sessions.push(Session {
            start_at,
            temperatures,
        })
    }

    if let Some(session) = sessions.iter().last().filter(|_| remove_old) {
        file.seek(std::io::SeekFrom::Start(0))?;
        file.set_len(0)?;
        file.write_all(&session.start_at.to_be_bytes())?;
        for temperature in session.temperatures.iter() {
            file.write_all(&[*temperature])?;
        }
    }

    Ok(sessions)
}

fn file() -> Result<File> {
    crate::fs::create_and_open_file(FILE)
}

fn time() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
