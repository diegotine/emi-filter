//! Parâmetros complementares. Zero em parasitas significa a aproximação ideal.
//! Limites e leituras de instrumento são cadastro, não um modelo de saturação.
use num_complex::Complex64 as C;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::f64::consts::PI;

pub const COMPONENTS: [&str; 12] = [
    "Cx1", "Cx2", "Cx3", "Cy1", "Cy2", "Bypass", "L1", "L2", "L3", "Rd", "R1", "R2",
];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Extra {
    pub servo_count: usize,
    pub noise_duty_percent: f64,
    pub l2_l3_k: f64,
    pub l1_interwinding_pf: f64,
    pub l1_core_pe_pf: f64,
    pub derivative_weights_enabled: bool,
    pub derivative_c0: f64,
    pub derivative_c1_s: f64,
    pub derivative_c2_s2: f64,
    // Exemplo documentado, desativado até duração e amortecimento serem informados.
    pub ringing_enabled: bool,
    pub ringing_frequency_hz: f64,
    pub ringing_interval_s: f64,
    pub ringing_relative_peak: f64,
    pub ringing_duration_s: Option<f64>,
    pub ringing_decay_s: Option<f64>,
    pub components: BTreeMap<String, Component>,
}
impl Default for Extra {
    fn default() -> Self {
        Self {
            servo_count: 3,
            noise_duty_percent: 50.0,
            l2_l3_k: 0.0,
            l1_interwinding_pf: 0.0,
            l1_core_pe_pf: 0.0,
            derivative_weights_enabled: false,
            derivative_c0: 1.0,
            derivative_c1_s: 0.0,
            derivative_c2_s2: 0.0,
            ringing_enabled: false,
            ringing_frequency_hz: 50000.0,
            ringing_interval_s: 0.0001,
            ringing_relative_peak: 5.0,
            ringing_duration_s: None,
            ringing_decay_s: None,
            components: COMPONENTS
                .into_iter()
                .map(|id| {
                    let mut c = Component::default();
                    if matches!(id, "Cx1" | "Cx2" | "Cx3") {
                        c.rated_v_ac = Some(275.0);
                    }
                    if matches!(id, "L2" | "L3") {
                        c.measurement_hz = Some(100000.0);
                        c.measured_q = Some(5.5);
                        c.measured_z_ohm = Some(0.680);
                    }
                    (id.into(), c)
                })
                .collect(),
        }
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Component {
    pub leakage_mohm: f64,
    pub series_l_nh: f64,
    pub parallel_c_pf: f64,
    pub alpha_per_v: f64,
    pub beta_per_v2: f64,
    pub tolerance_percent: Option<f64>,
    pub rated_v_ac: Option<f64>,
    pub rated_v_dc: Option<f64>,
    pub rated_rms_a: Option<f64>,
    pub rated_peak_a: Option<f64>,
    pub saturation_a: Option<f64>,
    pub rated_power_w: Option<f64>,
    pub temperature_c: Option<f64>,
    pub tcr_ppm_k: Option<f64>,
    pub measurement_hz: Option<f64>,
    pub measured_q: Option<f64>,
    pub measured_z_ohm: Option<f64>,
    pub measured_dcr_mohm: Option<f64>,
    pub lcr_equivalent_c_uf: Option<f64>,
    pub self_resonance_hz: Option<f64>,
    pub test_amplitude_v: Option<f64>,
    pub bias_v: Option<f64>,
    pub bias_a: Option<f64>,
    pub notes: String,
    pub use_z_curve: bool,
    pub z_curve: Vec<ZPoint>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ZPoint {
    pub frequency_hz: f64,
    pub re_ohm: f64,
    pub im_ohm: f64,
}

impl Extra {
    pub fn component(&self, id: &str) -> &Component {
        // validate() checks the fixed topology before the solver accesses the map.
        &self.components[id]
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.servo_count == 0
            || !self.noise_duty_percent.is_finite()
            || !(0.0..100.0).contains(&self.noise_duty_percent)
            || self.noise_duty_percent == 0.0
        {
            return Err(
                "Informe quantidade de servos positiva e duty estritamente entre 0 e 100%.".into(),
            );
        }
        if !self.l2_l3_k.is_finite() || self.l2_l3_k.abs() >= 1.0 {
            return Err(
                "Acoplamento L2/L3: use −1 < k < 1; k=0 mantém bobinas independentes.".into(),
            );
        }
        for (name, v) in [
            ("C entre enrolamentos", self.l1_interwinding_pf),
            ("C L1–PE", self.l1_core_pe_pf),
        ] {
            if !v.is_finite() || v < 0.0 {
                return Err(format!("{name}: valor não negativo e finito necessário."));
            }
        }
        for v in [
            self.derivative_c0,
            self.derivative_c1_s,
            self.derivative_c2_s2,
        ] {
            if !v.is_finite() {
                return Err("Pesos das derivadas devem ser finitos.".into());
            }
        }
        if self.derivative_weights_enabled
            && self.derivative_c0 == 0.0
            && self.derivative_c1_s == 0.0
            && self.derivative_c2_s2 == 0.0
        {
            return Err("A soma das bases não pode ter todos os pesos nulos.".into());
        }
        for id in COMPONENTS {
            let c = self
                .components
                .get(id)
                .ok_or_else(|| format!("Falta o cadastro do componente {id}."))?;
            for v in [c.leakage_mohm, c.series_l_nh, c.parallel_c_pf] {
                if !v.is_finite() || v < 0.0 {
                    return Err(format!(
                        "{id}: parasitas devem ser finitos e não negativos."
                    ));
                }
            }
            let capacitor = id.starts_with('C') || id == "Bypass";
            let resistor = matches!(id, "Rd" | "R1" | "R2");
            if (!capacitor && c.leakage_mohm != 0.0)
                || (!resistor && (c.series_l_nh != 0.0 || c.parallel_c_pf != 0.0))
            {
                return Err(format!(
                    "{id}: parâmetro adicional incompatível com o tipo do componente."
                ));
            }
            if c.alpha_per_v != 0.0 || c.beta_per_v2 != 0.0 {
                return Err(format!("{id}: C(V)/R(V) está cadastrado, mas o solver é linear. Use α=β=0 para calcular; coeficientes não nulos exigem solver não linear."));
            }
            if c.use_z_curve {
                if id == "L1" || (matches!(id, "L2" | "L3") && self.l2_l3_k != 0.0) {
                    return Err(format!("{id}: uma tabela escalar Z(f) não identifica os enrolamentos acoplados. Use R/L/Cp e k ou um modelo matricial medido."));
                }
                if c.z_curve.len() < 2 {
                    return Err(format!("{id}: Z(f) precisa de pelo menos dois pontos."));
                }
                let mut prev = 0.0;
                for point in &c.z_curve {
                    if !point.frequency_hz.is_finite()
                        || point.frequency_hz <= prev
                        || !point.re_ohm.is_finite()
                        || point.re_ohm < 0.0
                        || !point.im_ohm.is_finite()
                    {
                        return Err(format!("{id}: Z(f) exige frequências positivas crescentes, Re ≥ 0 e valores finitos."));
                    }
                    prev = point.frequency_hz;
                }
            }
        }
        if self.components.len() != COMPONENTS.len() {
            return Err("Cadastro contém componente desconhecido.".into());
        }
        if self.ringing_enabled {
            let duration = self
                .ringing_duration_s
                .ok_or("Ringing: informe a duração do pacote.")?;
            let decay = self
                .ringing_decay_s
                .ok_or("Ringing: informe a constante de decaimento.")?;
            for v in [
                duration,
                decay,
                self.ringing_interval_s,
                self.ringing_frequency_hz,
                self.ringing_relative_peak,
            ] {
                if !v.is_finite() || v <= 0.0 {
                    return Err("Ringing: parâmetros devem ser positivos e finitos.".into());
                }
            }
            if duration > self.ringing_interval_s {
                return Err(
                    "A duração do ringing não pode superar o intervalo entre pacotes.".into(),
                );
            }
        }
        Ok(())
    }
    pub fn fundamental(&self, square_hz: f64) -> f64 {
        if self.ringing_enabled {
            1.0 / self.ringing_interval_s
        } else {
            square_hz
        }
    }
    /// Fasor de pico da componente AC; a média DC da excitação é omitida.
    pub fn harmonic(&self, n: usize, amplitude: f64, f0: f64) -> C {
        let omega = 2.0 * PI * n as f64 * f0;
        let base = if self.ringing_enabled {
            let duration = self.ringing_duration_s.unwrap_or(0.0);
            let decay = self.ringing_decay_s.unwrap_or(1.0);
            let wc = 2.0 * PI * self.ringing_frequency_hz;
            let integral = |s: C| {
                if s.norm() < 1e-15 {
                    C::new(duration, 0.0)
                } else {
                    ((s * duration).exp() - C::new(1.0, 0.0)) / s
                }
            };
            (integral(C::new(-1.0 / decay, wc - omega))
                - integral(C::new(-1.0 / decay, -wc - omega)))
                * C::new(0.0, -amplitude * self.ringing_relative_peak * f0)
        } else if self.noise_duty_percent == 50.0 {
            if n.is_multiple_of(2) {
                C::new(0.0, 0.0)
            } else {
                C::new(0.0, -4.0 * amplitude / (PI * n as f64))
            }
        } else {
            C::new(0.0, -2.0 * amplitude / (PI * n as f64))
                * (C::new(1.0, 0.0)
                    - C::from_polar(1.0, -2.0 * PI * n as f64 * self.noise_duty_percent / 100.0))
        };
        let weight = if self.derivative_weights_enabled {
            C::new(
                self.derivative_c0 - omega * omega * self.derivative_c2_s2,
                omega * self.derivative_c1_s,
            )
        } else {
            C::new(1.0, 0.0)
        };
        base * weight
    }
}
impl Component {
    /// Substitui o ramo completo; sem extrapolação fora da faixa medida.
    pub fn measured_z(&self, f: f64) -> Result<Option<C>, String> {
        if !self.use_z_curve {
            return Ok(None);
        }
        let first = self.z_curve.first().ok_or("Tabela Z(f) vazia.")?;
        let last = self.z_curve.last().ok_or("Tabela Z(f) vazia.")?;
        if f < first.frequency_hz * (1.0 - 1e-12) || f > last.frequency_hz * (1.0 + 1e-12) {
            return Err(format!(
                "Z(f): {f:.3} Hz fora da faixa medida [{:.3}, {:.3}] Hz. Extrapolação desativada.",
                first.frequency_hz, last.frequency_hz
            ));
        }
        let i = self
            .z_curve
            .partition_point(|p| p.frequency_hz < f)
            .clamp(1, self.z_curve.len() - 1);
        let a = &self.z_curve[i - 1];
        let b = &self.z_curve[i];
        let t = ((f.ln() - a.frequency_hz.ln()) / (b.frequency_hz.ln() - a.frequency_hz.ln()))
            .clamp(0.0, 1.0);
        let z = C::new(
            a.re_ohm + (b.re_ohm - a.re_ohm) * t,
            a.im_ohm + (b.im_ohm - a.im_ohm) * t,
        );
        if z.norm() < 1e-25 {
            return Err("Z(f) nula: curto ideal não suportado neste ramo.".into());
        }
        Ok(Some(z))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rectangular_and_ringing_phasors_match_numerical_fourier_integrals() {
        let mut p = Extra {
            noise_duty_percent: 37.0,
            ..Extra::default()
        };
        for ringing in [false, true] {
            p.ringing_enabled = ringing;
            p.ringing_duration_s = Some(0.000062);
            p.ringing_decay_s = Some(0.000018);
            p.validate().unwrap();
            let f0 = p.fundamental(20000.0);
            let samples = 200000;
            for n in [1, 2, 3, 7, 15] {
                let mut integral = C::new(0.0, 0.0);
                for i in 0..samples {
                    let t = (i as f64 + 0.5) / (samples as f64 * f0);
                    let v = if ringing {
                        if t < p.ringing_duration_s.unwrap() {
                            p.ringing_relative_peak
                                * (-t / p.ringing_decay_s.unwrap()).exp()
                                * (2.0 * PI * p.ringing_frequency_hz * t).sin()
                        } else {
                            0.0
                        }
                    } else if t * f0 < p.noise_duty_percent / 100.0 {
                        1.0
                    } else {
                        -1.0
                    };
                    integral +=
                        C::from_polar(v, -2.0 * PI * f0 * n as f64 * t) * 2.0 / samples as f64;
                }
                assert!(
                    (integral - p.harmonic(n, 1.0, f0)).norm() < 2e-8,
                    "ringing={ringing}, n={n}: {integral}"
                );
            }
        }
        p.ringing_enabled = false;
        p.derivative_weights_enabled = true;
        p.derivative_c0 = 0.0;
        p.derivative_c1_s = 1e-6;
        let mut baseline = p.clone();
        baseline.derivative_weights_enabled = false;
        let actual = p.harmonic(3, 1.0, 20000.0);
        let expected = baseline.harmonic(3, 1.0, 20000.0) * C::new(0.0, 2.0 * PI * 60000.0 * 1e-6);
        assert!((actual - expected).norm() < 1e-12);
    }
    #[test]
    fn impedance_interpolation_is_bounded_and_nonlinearity_cannot_be_ignored() {
        let mut p = Extra::default();
        let c = p.components.get_mut("Cx1").unwrap();
        c.alpha_per_v = 1e-6;
        assert!(p.validate().unwrap_err().contains("linear"));
        let c = p.components.get_mut("Cx1").unwrap();
        c.alpha_per_v = 0.0;
        c.use_z_curve = true;
        c.z_curve = vec![
            ZPoint {
                frequency_hz: 1000.0,
                re_ohm: 2.0,
                im_ohm: -100.0,
            },
            ZPoint {
                frequency_hz: 100000.0,
                re_ohm: 4.0,
                im_ohm: -10.0,
            },
        ];
        let z = c.measured_z(10000.0).unwrap().unwrap();
        assert!((z - C::new(3.0, -55.0)).norm() < 1e-12);
        assert!(c.measured_z(999.0).is_err());
        assert!(c.measured_z(100001.0).is_err());
        p.validate().unwrap();
    }
}
