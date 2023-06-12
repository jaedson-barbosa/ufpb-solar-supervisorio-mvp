// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use modbus::{tcp::Transport, Client};
use std::sync::{Arc, Mutex};

struct State {
    inverter_transport: Arc<Mutex<Transport>>,
    ncu_transport: Arc<Mutex<Transport>>,
    weather_station_transport: Arc<Mutex<Transport>>,
}

#[derive(serde::Serialize)]
struct WeatherData {
    wind_speed: f32,
    wind_direction: f32,
    relative_humidity: f32,
    temperature: f32,
    atmospheric_pressure: f32,
    dhi: f32,
    ghi: f32,
    dni: f32,
    precipitation: f32,
    gti: f32,
}

fn read_f32(transport: &mut Transport, address: u16, quantity: u16) -> Vec<f32> {
    transport
        .read_holding_registers(address, quantity * 2)
        .unwrap()
        .chunks_exact(2)
        .map(|v| {
            let a = v[0].to_be_bytes();
            let b = v[1].to_be_bytes();
            [a[0], a[1], b[0], b[1]]
        })
        .map(|v| f32::from_be_bytes(v))
        .collect::<Vec<f32>>()
}

fn read_i32(transport: &mut Transport, address: u16, quantity: u16) -> Vec<i32> {
    transport
        .read_holding_registers(address, quantity * 2)
        .unwrap()
        .chunks_exact(2)
        .map(|v| {
            let a = v[0].to_be_bytes();
            let b = v[1].to_be_bytes();
            [a[0], a[1], b[0], b[1]]
        })
        .map(|v| i32::from_be_bytes(v))
        .collect::<Vec<i32>>()
}

fn read_i16(transport: &mut Transport, address: u16, quantity: u16) -> Vec<i16> {
    transport
        .read_holding_registers(address, quantity)
        .unwrap()
        .iter()
        .map(|&v| i16::from_be_bytes(v.to_be_bytes()))
        .collect::<Vec<i16>>()
}

fn read_u16(transport: &mut Transport, address: u16, quantity: u16) -> Vec<u16> {
    transport.read_holding_registers(address, quantity).unwrap()
}

fn read_u8(transport: &mut Transport, address: u16, quantity: u16) -> Vec<u8> {
    transport
        .read_holding_registers(address, quantity / 2)
        .unwrap()
        .iter()
        .flat_map(|&v| v.to_be_bytes())
        .collect::<Vec<u8>>()
}

#[tauri::command]
fn request_weather_data(state: tauri::State<State>) -> WeatherData {
    let mut transport = state.weather_station_transport.lock().unwrap();
    let mut read_weather = |index| -> Vec<f32> { read_f32(&mut transport, index, 1) };

    WeatherData {
        wind_speed: read_weather(119)[0],
        wind_direction: read_weather(121)[0],
        relative_humidity: read_weather(15)[0],
        temperature: read_weather(25)[0],
        atmospheric_pressure: read_weather(35)[0],
        dhi: read_weather(45)[0],
        ghi: read_weather(55)[0],
        dni: read_weather(65)[0],
        precipitation: read_weather(123)[0],
        gti: read_weather(79)[0],
    }
}

#[derive(serde::Serialize)]
struct TrackerData {
    angle: f32,
    motor_current: u16,
    target_angle: f32,
    temperature: i16,
    state_of_charge: u8,
}

fn request_tracker_data(transport: &mut Transport, tracker_index: u16) -> TrackerData {
    let angle_addr = tracker_index * 23 + 30_152;
    let motor_current_addr = angle_addr + 2;
    let target_angle_addr = angle_addr + 4;
    let temperature_addr = angle_addr + 14;
    let state_of_charge_addr = angle_addr + 7;

    TrackerData {
        angle: read_f32(transport, angle_addr, 1)[0],
        motor_current: read_u16(transport, motor_current_addr, 1)[0],
        target_angle: read_f32(transport, target_angle_addr, 1)[0],
        temperature: read_i16(transport, temperature_addr, 1)[0],
        state_of_charge: read_u8(transport, state_of_charge_addr, 2)[1],
    }
}

#[tauri::command]
fn request_trackers_data(state: tauri::State<State>) -> [TrackerData; 4] {
    let mut ncu_transport = state.ncu_transport.lock().unwrap();
    [
        request_tracker_data(&mut ncu_transport, 0),
        request_tracker_data(&mut ncu_transport, 1),
        request_tracker_data(&mut ncu_transport, 2),
        request_tracker_data(&mut ncu_transport, 3),
    ]
}

#[derive(serde::Serialize)]
struct InverterData {
    number_of_string: u16,
    input_power: i32,
    active_power: i32,
    reactive_power: i32,
    power_factor: i16,
    efficiency: u16,
    temperature: i16,
    pv_voltage_current: [i16; 24],
}

fn request_inverter_data(transport: &mut Transport) -> InverterData {
    InverterData {
        number_of_string: read_u16(transport, 30071, 1)[0],
        input_power: read_i32(transport, 32064, 1)[0],
        active_power: read_i32(transport, 32080, 1)[0],
        reactive_power: read_i32(transport, 32082, 1)[0],
        power_factor: read_i16(transport, 32084, 1)[0],
        efficiency: read_u16(transport, 32086, 1)[0],
        temperature: read_i16(transport, 32087, 1)[0],
        pv_voltage_current: read_i16(transport, 32016, 24)
            .try_into()
            .expect("wrong size"),
    }
}

#[tauri::command]
fn request_inverters_data(state: tauri::State<State>) -> [InverterData; 2] {
    let mut inverter_transport = state.inverter_transport.lock().unwrap();
    inverter_transport.set_uid(1);
    let inverter_1_data = request_inverter_data(&mut inverter_transport);
    inverter_transport.set_uid(2);
    let inverter_2_data = request_inverter_data(&mut inverter_transport);
    [inverter_1_data, inverter_2_data]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracker_load() {
        let mut ncu_transport = Transport::new("192.168.7.101").unwrap();
        let data = request_tracker_data(&mut ncu_transport, 0);
        assert!(data.angle != 0.0);
    }

    #[test]
    fn inverter_load() {
        let mut inverter_transport = Transport::new("192.168.7.111").unwrap();
        let data = request_inverter_data(&mut inverter_transport);
        assert!(data.number_of_string > 0);
    }
}

fn main() {
    let inverter_transport = Transport::new("192.168.7.111").unwrap();
    let ncu_transport = Transport::new("192.168.7.101").unwrap();
    let weather_station_transport = Transport::new("192.168.7.105").unwrap();

    let state = State {
        inverter_transport: Arc::new(Mutex::new(inverter_transport)),
        ncu_transport: Arc::new(Mutex::new(ncu_transport)),
        weather_station_transport: Arc::new(Mutex::new(weather_station_transport)),
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            request_weather_data,
            request_trackers_data,
            request_inverters_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
