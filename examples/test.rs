#![feature(impl_trait_in_assoc_type)]
use core::fmt::Write as _;

use clap::Parser;
use embassy_executor::{Executor, Spawner};
use embassy_net::tcp::TcpSocket;
use embassy_net::{Config, Ipv4Address, Ipv4Cidr, StackResources};
use embassy_net_tuntap::TunTapDevice;
use embassy_time::{Duration, Timer};
use embedded_io_async::Write as _;
use heapless::Vec;
use log::*;
use rand_core::{OsRng, RngCore};
use static_cell::StaticCell;

#[derive(Parser)]
#[clap(version = "1.0")]
struct Opts {
    /// TAP device name
    #[clap(long, default_value = "tap0")]
    tap: String,
    /// use a static IP instead of DHCP
    #[clap(long)]
    static_ip: bool,
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, TunTapDevice>) -> ! {
    runner.run().await
}

#[derive(Default)]
struct StrWrite(pub heapless::Vec<u8, 256>);

impl core::fmt::Write for StrWrite {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        self.0.extend_from_slice(s.as_bytes()).unwrap();
        Ok(())
    }
}

#[embassy_executor::task]
async fn main_task(spawner: Spawner) {
    let opts: Opts = Opts::parse();

    // Init network device
    let device = TunTapDevice::new(&opts.tap).unwrap();

    // Choose between dhcp or static ip
    let config = if opts.static_ip {
        Config::ipv4_static(embassy_net::StaticConfigV4 {
            address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 69, 2), 24),
            dns_servers: Vec::new(),
            gateway: Some(Ipv4Address::new(192, 168, 69, 1)),
        })
    } else {
        Config::dhcpv4(Default::default())
    };

    // Generate random seed
    let mut seed = [0; 8];
    OsRng.fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);

    // Init network stack
    static RESOURCES: StaticCell<StackResources<4>> = StaticCell::new();
    let (stack, runner) =
        embassy_net::new(device, config, RESOURCES.init(StackResources::new()), seed);
    spawner.spawn(http_task(stack)).unwrap();
    // Launch network task
    spawner.spawn(net_task(runner)).unwrap();
}

#[embassy_executor::task(pool_size = 8)]
async fn http_task(stack: embassy_net::Stack<'static>) {
    // Then we can use it!
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));
        info!("Listening on TCP:80...");
        if let Err(_) = socket.accept(80).await {
            warn!("accept error");
            continue;
        }
        info!("Accepted a connection");

        // Write some quick output
        let mut w = StrWrite::default();
        let body = "<html><head><title>title</title></head><body><h1>hello</h1></body></html>";
        write!(w, "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\n\r\n{}", body.len(), body).unwrap();
        let r = socket.write_all(&w.0).await;
        if let Err(e) = r {
            warn!("write error: {:?}", e);
            return;
        }

        _ = socket.flush().await;
        info!("Closing the connection");
        socket.close();
        info!("Flushing the RST out...");
        _ = socket.flush().await;
        info!("Finished with the socket");
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("async_io", log::LevelFilter::Info)
        .format_timestamp_nanos()
        .init();

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(main_task(spawner)).unwrap();
    });
}
