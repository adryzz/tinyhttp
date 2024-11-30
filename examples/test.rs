#![feature(impl_trait_in_assoc_type)]
#![feature(generic_arg_infer)]

use clap::Parser;
use embassy_executor::{Executor, Spawner};
use embassy_net::{Config, Ipv4Address, Ipv4Cidr, StackResources};
use embassy_net_tuntap::TunTapDevice;
use heapless::Vec;
use rand_core::{OsRng, RngCore};
use static_cell::StaticCell;
use tinyhttp::config::HttpConfig;
use tinyhttp::error::Error;
use tinyhttp::reader::RequestReader;
use tinyhttp::status::StatusCode;
use tinyhttp::writer::{HttpResponse, ResponseWriter};
use tinyhttp::{router, HttpServer};

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
    let config = HttpConfig::default();

    HttpServer::<1024, 1024>::new(stack, &config)
        .route(router! {
            "/" => send_204,
        })
        .run()
        .await
}

async fn send_204<'a, 'b, 'c>(
    _reader: RequestReader<'a, 'b, 'c>,
    writer: ResponseWriter<'a, 'b>,
) -> Result<HttpResponse, Error> {
    writer
        .start(StatusCode::NO_CONTENT)
        .await?
        .body_empty()
        .await
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
