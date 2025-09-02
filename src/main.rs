// ESP-IDF specific imports
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;

use esp_idf_svc::io::EspIOError;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys::link_patches;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};

// Solana related imports
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::pubkey::Pubkey;
use solana_system_interface::instruction as system_instruction;
use solana_transaction::Transaction;
use solana_keypair::{Keypair, Signer};

use log::info;

mod solrpc;
use crate::solrpc::{get_latest_blockhash, send_transaction};



fn main() -> Result<(), EspIOError> {
    link_patches();
    EspLogger::initialize_default();

    // WiFi initialization
    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut esp_wifi = EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs)).unwrap();
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sys_loop.clone()).unwrap();

    wifi.set_configuration(&esp_idf_svc::wifi::Configuration::Client(
        esp_idf_svc::wifi::ClientConfiguration {
            ssid: "berg_iot".try_into().unwrap(), // WiFi SSID
            password: "bergiotsupersecret123.".try_into().unwrap(), // WiFi password
            auth_method: esp_idf_svc::wifi::AuthMethod::WPA2Personal,
            ..Default::default()
        },
    ))
    .unwrap();

    wifi.start().unwrap();
    wifi.connect().unwrap();
    wifi.wait_netif_up().unwrap();

    let keypair = Keypair::new();
    info!("Keyapir generated for demo: {}", keypair.pubkey());

    loop {
        unsafe {
            // Sleep for 2 seconds with each iteration
            esp_idf_svc::sys::sleep(2);
        }

        if let Ok(blockhash) = get_latest_blockhash() {
            info!("Latest blockhash: {}", blockhash);

            // Example: Build and sign a transaction
            let to_pubkey = Pubkey::new_unique();
            let from_pubkey = keypair.pubkey();
            // Transfer 1 sol
            let instruction = system_instruction::transfer(&from_pubkey, &to_pubkey, LAMPORTS_PER_SOL);

            let transaction = Transaction::new_signed_with_payer(
                &[instruction], 
                Some(&from_pubkey),
                &[&keypair],
                blockhash
            );
            
            info!("Signed transaction: {:?}", transaction);

            // Send the transaction to the Solana network
            match send_transaction(&transaction) {
                Ok(signature) => {
                    info!("✅ Transaction sent successfully!");
                    info!("📋 Transaction signature: {}", signature);
                }
                Err(e) => {
                    info!("❌ Failed to send transaction: {}", e);
                }
            }
        } else {
            info!("Failed to get blockhash");
        }

    }
}

