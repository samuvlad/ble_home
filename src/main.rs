use core::time;
use std::str::FromStr;
use std::{str, thread};

use async_recursion::async_recursion;
use bluer::{Address, Device};

async fn read_char(device: Device) {
    let service_uuid = uuid::Uuid::parse_str("0000ffe0-0000-1000-8000-00805f9b34fb").unwrap();
    let characteristics_uuid =
        uuid::Uuid::parse_str("0000ffe1-0000-1000-8000-00805f9b34fb").unwrap();

    println!("    Device provides our service!");

    println!("    Connecting... {}", device.address());
    let mut retries = 2;
    loop {
        match device.connect().await {
            Ok(()) => break,
            Err(err) if retries > 0 => {
                println!("    Connect error: {}", &err);
                retries -= 1;
            }
            Err(_) => {
                println!("Error to connect");
                retries = 6;
            }
        }
        thread::sleep(time::Duration::from_millis(1000));
    }
    println!("Connected");
    let mut current_data = Vec::new();
    let service = device.services().await;
    match service {
        Ok(service) => {
            for service in service {
                let uuid = service.uuid().await.unwrap();

                if uuid == service_uuid {
                    loop {
                        for char in service.characteristics().await.unwrap() {
                            let uuid = char.uuid().await.unwrap();
                            if uuid == characteristics_uuid {
                                let udata = char.read().await;
                                match udata {
                                    Ok(udata) => {
                                        let data =
                                            str::from_utf8(&udata).unwrap_or("Data is not char");
                                        if current_data != udata {
                                            println!("Datos: {}", data);
                                        }

                                        current_data = udata.clone();
                                    }
                                    Err(_) => reconect(device.to_owned()).await,
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(_) => println!("Error to get services"),
    }
    device.disconnect().await.unwrap();
}

#[tokio::main]
async fn main() {
    let session = bluer::Session::new().await.unwrap();
    let adapter = session.default_adapter().await.unwrap();

    adapter.set_powered(true).await.unwrap();
    let address: Address = Address::from_str("01:23:45:67:93:BB").unwrap();

    let device = adapter.device(address).unwrap();
    read_char(device).await;
}
#[async_recursion]
async fn reconect(device: Device) {
    read_char(device).await;
}
