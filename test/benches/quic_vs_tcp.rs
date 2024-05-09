use connection::{quic_connection, tcp_connection};
use criterion::{criterion_group, criterion_main};
// use transmission::{quic_data_recv, tcp_data_recv};

criterion_group!(
    benches,
    quic_connection,
    tcp_connection,
    // quic_data_recv,
    // tcp_data_recv,
);
criterion_main!(benches);

mod connection;
mod transmission;
