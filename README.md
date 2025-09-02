# REsp32Sol - ESP32 Solana DePIN Project

A decentralized physical infrastructure (DePIN) project that connects ESP32 microcontrollers to the Solana blockchain. This project demonstrates how to run Solana operations on embedded hardware, including transaction creation, signing, and blockchain interaction. The special part is that it's "std" friendly, using esp_idf_svc provides us an std like environment, helping in writing more complex programs that might require threading, networking on other high level concepts. This environment also allows us to use official solana crates for transaction creation, signing, keypair management and more The no-std environment limits us to core Rust without allocations, threads, or I/O from std. The trade-off is that std binaries are larger and rely on ESP-IDF's initialization (e.g., the link_patches() to fix possible runtime issues).

## 🌟 Features

- **WiFi Connectivity**: Connects to WiFi networks for communication
- **Solana Integration**: Makes RPC calls to Solana networks
- **Transaction Creation & Sending**: Creates, signs, and sends Solana transactions on-device
- **Ed25519 Cryptography**: Secure key generation and transaction signing on-device
- **Low Power**: Optimized for battery-powered IoT devices, further optimization possible with low power mode

## 🔧 Hardware Requirements

- **ESP32-C3 or ESP32-S3** microcontroller if you want to follow this porject, other boards require similar setup
- **USB connection** for flashing and monitoring, through usb cable
- **WiFi network** access for blockchain communication and others
- **4MB+ flash memory** (configured in `partitions.csv` for this project for the Esp32 c3 supermini with 4MB flash memory)

## 📋 Prerequisites

Before setting up this project, ensure you have:

### System Requirements
- **macOS** (tested on macOS with Apple Silicon)
- **Homebrew** package manager
- **Rust** toolchain (latest stable version)
- **Git** for version control

### Network Requirements
- Solana RPC endpoints (devnet/mainnet) as needed
- WiFi network for ESP32 connectivity

## 🚀 Setup Instructions

### Step 1: Install ESP32 Development Tools for rust

```bash
# Install ESP flashing tools
cargo install cargo-espflash espflash

# Install USB library for device communication
brew install libusb

# Install ESP toolchain manager
cargo install espup --locked
```

### Step 2: Setup ESP Environment

```bash
# Setup ESP development environment
espup install

# Add ESP environment alias to your shell profile
echo "alias get_esprs='. $HOME/export-esp.sh'" >> ~/.zshrc
source ~/.zshrc

# Activate ESP environment, it's needed in all shells that build this esp project
get_esprs
```

### Step 3: Install Project Dependencies

```bash
# Install project generation tools
cargo install cargo-generate ldproxy

# Install additional Rust targets for ESP32
rustup target add riscv32imc-unknown-none-elf
```

### Step 4: Generate and Configure Project

```bash
# Generate a new ESP-IDF project (alternative to cloning this repo) for std like environment on esp32
# During project generation, you will be prompted to choose what board is being used, in this case it was esp32c3
cargo generate esp-rs/esp-idf-template REsp32Sol
```

### Step 5: Configure Project Settings

#### WiFi Configuration
Edit the WiFi credentials in `src/main.rs`:

```rust
let ssid = "YOUR_WIFI_SSID".try_into().unwrap();
let password = "YOUR_WIFI_PASSWORD".try_into().unwrap();
```

#### Network Configuration
Choose your Solana network in `src/main.rs`:

```rust
const RPC_URL: &str = "https://api.devnet.solana.com";  // For testing
// const RPC_URL: &str = "https://api.mainnet-beta.solana.com";  // For production
```

## 🛠️ Building and Flashing

### Build the Project

```bash
# Build in release mode for production
cargo build --release

# Or build in debug mode for development
cargo build
```

### Flash to ESP32

```bash
# Flash and start monitoring (replace esp32c3 with your ESP32 variant)
cargo espflash flash --chip esp32c3 --monitor

# Flash without monitoring
cargo espflash flash --chip esp32c3

# Flash with specific serial port
cargo espflash flash --chip esp32c3 --monitor /dev/tty.usbserial-XXXX
```

### Monitor Serial Output

```bash
# Monitor device output without flashing
cargo espflash monitor --chip esp32c3

# Or use the flash command with --monitor flag
cargo espflash flash --chip esp32c3 --monitor
```

## 📁 Project Structure

```
REsp32Sol/
├── src/
│   └── main.rs              # Main application logic
├── sdkconfig.defaults       # ESP-IDF configuration
├── partitions.csv           # Flash partition table
├── Cargo.toml              # Rust dependencies
├── build.rs                # Build script
├── rust-toolchain.toml     # Rust toolchain configuration
└── README.md               # This file
```

## ⚙️ Configuration Files that can be customized for your device

### sdkconfig.defaults
Contains ESP-IDF configuration including:
- Custom partition table settings
- Flash size configuration (4MB), depends on device memory
- ESP32-specific optimizations

### partitions.csv
Custom partition table allocating:
- 3MB for application code
- 0.6KB for NVS (Non-Volatile Storage)
- 1KB for PHY calibration data

## 🔧 Customization

### Changing Solana Network

```rust
// In src/main.rs, modify the RPC_URL constant
const RPC_URL: &str = "https://api.mainnet-beta.solana.com"; // Mainnet
// const RPC_URL: &str = "https://api.testnet.solana.com";     // Testnet
```

### Adjusting Monitoring Interval

```rust
// Modify the sleep duration in the main loop
unsafe {
    esp_idf_svc::sys::sleep(5); // Change from 2 to 5 seconds
}
```

### Adding New RPC Methods

Extend the `SolanaRpcMethod` enum and implement the corresponding methods:

```rust
pub enum SolanaRpcMethod {
    // ... existing methods
    GetAccountBalance(String), // New method
}

// Implement the required methods
impl SolanaRpcMethod {
    pub fn method_name(&self) -> &'static str {
        match self {
            // ... existing cases
            SolanaRpcMethod::GetAccountBalance(_) => "getBalance",
        }
    }

    pub fn params(&self) -> serde_json::Value {
        match self {
            // ... existing cases
            SolanaRpcMethod::GetAccountBalance(account) => {
                json!([account])
            }
        }
    }
}
```

## 🔍 Monitoring and Debugging

### Serial Output, can be accessed by using the --monitor flag while using espflash 
The device logs comprehensive information:
- ✅ WiFi connection status
- ✅ Blockhash retrieval
- ✅ Transaction creation and signing
- ❌ Error messages with context
- 🔄 System status updates

## 🐛 Troubleshooting

### Common Issues

#### 1. Flash Size Too Small
```
Error: espflash::image_too_big
```
**Solution**: Ensure your ESP32 has at least 4MB flash memory and the partition table is configured correctly. Or that you application size is smaller, depending on your device storage.

#### 2. WiFi Connection Failed
```
❌ WiFi Connection Failed: Failed to connect to WiFi network
```
**Solution**:
- Verify WiFi credentials in `src/main.rs`
- Check signal strength
- Ensure the network supports the ESP32's WiFi standard

#### 3. Solana RPC Errors
```
❌ Failed to retrieve blockhash: HTTP Connection Failed
```
**Solution**:
- Check internet connectivity
- Verify RPC endpoint is accessible
- Consider switching between devnet/mainnet/testnet

#### 4. Build Errors
```
error[E0463]: can't find crate for `esp_idf_sys`
```
**Solution**:
- Ensure ESP environment is activated: `get_esprs`
- Reinstall ESP toolchain: `espup install`

### Device Reset
If the device becomes unresponsive:
1. Press the reset button on your ESP32
2. Re-flash the firmware
3. Check power supply stability

## 📊 Performance Characteristics

- **Memory Usage**: ~2-3MB application footprint
- **CPU Usage**: Low power consumption during monitoring
- **Network**: Minimal bandwidth usage (JSON RPC calls)
- **Flash Wear**: Low write frequency to flash memory

## 🔒 Security Considerations

- **Key Storage**: Private keys are generated in RAM (consider secure storage for production)
- **Network Security**: Uses HTTPS for RPC communication
- **Input Validation**: All user inputs are validated
- **Error Handling**: Sensitive information is not logged

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test on ESP32 hardware
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙋 Support

For support and questions:
- Check the troubleshooting section above
- Review ESP32 and Solana documentation
- Open an issue on the repository

## 🔄 Version History

- **v0.1.0**: Initial release with basic Solana integration
- WiFi connectivity
- Transaction creation and signing
- Real-time blockhash monitoring

---

**Happy coding with ESP32 and Solana! 🚀**
