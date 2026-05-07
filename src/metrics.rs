use crate::speedtest::SpeedtestResult;
use prometheus::core::{Atomic, AtomicF64, AtomicI64, GenericGaugeVec};
use prometheus::{GaugeVec, IntGaugeVec, Opts, Registry};

pub type IntGauge = Gauge<AtomicI64>;
pub type FloatGauge = Gauge<AtomicF64>;

pub struct Gauge<T: Atomic>(GenericGaugeVec<T>);

pub fn register_int(registry: &Registry, name: &str, help: &str) -> IntGauge {
    let gauge =
        IntGaugeVec::new(Opts::new(name, help), &["server_name", "server_id", "isp"]).unwrap();
    registry.register(Box::new(gauge.clone())).unwrap();
    Gauge(gauge)
}

pub fn register(registry: &Registry, name: &str, help: &str) -> FloatGauge {
    let gauge =
        GaugeVec::new(Opts::new(name, help), &["server_name", "server_id", "isp"]).unwrap();
    registry.register(Box::new(gauge.clone())).unwrap();
    Gauge(gauge)
}

impl<T: Atomic> Gauge<T> {
    pub fn set(&self, value: T::T, speedtest_result: &SpeedtestResult) {
        let values = &[
            speedtest_result.server.name.as_str(),
            &format!("{}", speedtest_result.server.id),
            speedtest_result.isp.as_str(),
        ];
        self.0.with_label_values(values).set(value);
    }
}
