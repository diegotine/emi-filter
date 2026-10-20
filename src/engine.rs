//! Modelo fasorial nativo do filtro EMI, rede -> Cx1 -> L1 -> Rd/Cx2/Cy -> L2/L3 -> Cx3 -> servos.
//!
//! A excitação de ruído vem dos servos para a rede. A fundamental de alimentação
//! é um parâmetro independente: este modelo linear não sincroniza a portadora
//! com os seus zeros. Os valores de impedância da rede são módulos em 60 Hz;
//! L é obtido por sqrt(|Zref|²-R²)/(2πfref), e a curva usa R+j2πfL.
//! As perdas AC derivadas de Q não são DCR nem um modelo de aquecimento.

use num_complex::Complex64;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

type C = Complex64;
type V2 = [C; 2];
type M2 = [[C; 2]; 2];
const ZERO: C = C::new(0.0, 0.0);
const ONE: C = C::new(1.0, 0.0);

/// Unidades visíveis no nome de cada campo. Zero em C remove esse capacitor.
/// ESR/ESL e Cp iguais a zero são aproximações ideais, não medições.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Parameters {
    pub cx1_uf: f64,
    pub cx2_uf: f64,
    pub cx3_uf: f64,
    pub cy1_nf: f64,
    pub cy2_nf: f64,
    pub rd_ohm: f64,
    pub r1_kohm: f64,
    pub r2_kohm: f64,
    pub l1_sum_mh: f64,
    pub l1_opp_uh: f64,
    pub l1_dcr_mohm: f64,
    pub l1_q: f64,
    pub l1_q_hz: f64,
    pub l2_uh: f64,
    pub l3_uh: f64,
    pub l2_rs_ohm: f64,
    pub l3_rs_ohm: f64,
    pub grid_phase_z_ohm: f64,
    pub grid_z_ref_hz: f64,
    pub grid_phase_r_ohm: f64,
    pub grid_cm_z_ohm: f64,
    pub grid_cm_r_ohm: f64,
    pub source_phase_r_ohm: f64,
    pub source_cm_r_ohm: f64,
    pub pe_r_ohm: f64,
    pub pe_l_uh: f64,
    pub noise_frequency_hz: f64,
    pub noise_delay_deg: f64,
    pub noise_phase_a_deg: f64,
    pub noise_peak_v: f64,
    pub noise_max_hz: f64,
    pub sweep_min_hz: f64,
    pub sweep_max_hz: f64,
    pub mains_phase_v_rms: f64,
    pub mains_frequency_hz: f64,
    pub mains_phase_deg: f64,
    pub load_rms_a: f64,
    pub load_peak_a: f64,
    pub cx1_esr_ohm: f64,
    pub cx2_esr_ohm: f64,
    pub cx3_esr_ohm: f64,
    pub cy_esr_ohm: f64,
    pub cx1_esl_nh: f64,
    pub cx2_esl_nh: f64,
    pub cx3_esl_nh: f64,
    pub cy_esl_nh: f64,
    pub choke_cp_pf: f64,
    pub sweep_points: usize,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            cx1_uf: 1.5,
            cx2_uf: 0.47,
            cx3_uf: 0.33,
            cy1_nf: 3.3,
            cy2_nf: 3.3,
            rd_ohm: 4.7,
            r1_kohm: 150.0,
            r2_kohm: 150.0,
            l1_sum_mh: 1.045,
            l1_opp_uh: 8.8,
            l1_dcr_mohm: 22.0,
            l1_q: 40.0,
            l1_q_hz: 1000.0,
            l2_uh: 1.1,
            l3_uh: 1.1,
            l2_rs_ohm: 0.680 / (1.0_f64 + 5.5 * 5.5).sqrt(),
            l3_rs_ohm: 0.680 / (1.0_f64 + 5.5 * 5.5).sqrt(),
            grid_phase_z_ohm: 0.5,
            grid_z_ref_hz: 60.0,
            grid_phase_r_ohm: 0.0,
            grid_cm_z_ohm: 0.5,
            grid_cm_r_ohm: 0.0,
            source_phase_r_ohm: 50.0,
            source_cm_r_ohm: 1000.0,
            pe_r_ohm: 0.0,
            pe_l_uh: 0.0,
            noise_frequency_hz: 20000.0,
            noise_delay_deg: 120.0,
            noise_phase_a_deg: 37.0,
            noise_peak_v: 1.0,
            noise_max_hz: 300000.0,
            sweep_min_hz: 1000.0,
            sweep_max_hz: 300000.0,
            mains_phase_v_rms: 127.0,
            mains_frequency_hz: 60.0,
            mains_phase_deg: 120.0,
            load_rms_a: 5.0,
            load_peak_a: 20.0,
            cx1_esr_ohm: 0.0,
            cx2_esr_ohm: 0.0,
            cx3_esr_ohm: 0.0,
            cy_esr_ohm: 0.0,
            cx1_esl_nh: 0.0,
            cx2_esl_nh: 0.0,
            cx3_esl_nh: 0.0,
            cy_esl_nh: 0.0,
            choke_cp_pf: 0.0,
            sweep_points: 6001,
        }
    }
}

impl Parameters {
    /// Indutância equivalente por fase da rede (H).
    pub fn grid_phase_inductance_h(&self) -> f64 {
        (self.grid_phase_z_ohm.powi(2) - self.grid_phase_r_ohm.powi(2)).sqrt()
            / (2.0 * PI * self.grid_z_ref_hz)
    }

    /// Indutância da porta CM completa (H), independente dos ramos de fase.
    pub fn grid_cm_inductance_h(&self) -> f64 {
        (self.grid_cm_z_ohm.powi(2) - self.grid_cm_r_ohm.powi(2)).sqrt()
            / (2.0 * PI * self.grid_z_ref_hz)
    }

    pub fn grid_phase_impedance(&self, frequency_hz: f64) -> C {
        C::new(
            self.grid_phase_r_ohm,
            2.0 * PI * frequency_hz * self.grid_phase_inductance_h(),
        )
    }

    pub fn grid_dm_impedance(&self, frequency_hz: f64) -> C {
        self.grid_phase_impedance(frequency_hz) * 2.0
    }

    pub fn grid_cm_impedance(&self, frequency_hz: f64) -> C {
        C::new(
            self.grid_cm_r_ohm,
            2.0 * PI * frequency_hz * self.grid_cm_inductance_h(),
        )
    }

    pub fn line_voltage_rms(&self) -> f64 {
        2.0 * self.mains_phase_v_rms * (self.mains_phase_deg.to_radians() / 2.0).sin().abs()
    }

    pub fn validate(&self) -> Result<(), String> {
        let nonnegative = [
            ("Cx1", self.cx1_uf),
            ("Cx2", self.cx2_uf),
            ("Cx3", self.cx3_uf),
            ("Cy1", self.cy1_nf),
            ("Cy2", self.cy2_nf),
            ("Rd", self.rd_ohm),
            ("R1", self.r1_kohm),
            ("R2", self.r2_kohm),
            ("DCR de L1", self.l1_dcr_mohm),
            ("L2", self.l2_uh),
            ("L3", self.l3_uh),
            ("Rs de L2", self.l2_rs_ohm),
            ("Rs de L3", self.l3_rs_ohm),
            ("R da rede por fase", self.grid_phase_r_ohm),
            ("R da rede CM", self.grid_cm_r_ohm),
            ("R do PE", self.pe_r_ohm),
            ("L do PE", self.pe_l_uh),
            ("Tensão da alimentação", self.mains_phase_v_rms),
            ("Corrente RMS", self.load_rms_a),
            ("Corrente de pico", self.load_peak_a),
            ("ESR Cx1", self.cx1_esr_ohm),
            ("ESR Cx2", self.cx2_esr_ohm),
            ("ESR Cx3", self.cx3_esr_ohm),
            ("ESR Cy", self.cy_esr_ohm),
            ("ESL Cx1", self.cx1_esl_nh),
            ("ESL Cx2", self.cx2_esl_nh),
            ("ESL Cx3", self.cx3_esl_nh),
            ("ESL Cy", self.cy_esl_nh),
            ("Cp por enrolamento", self.choke_cp_pf),
        ];
        for (name, value) in nonnegative {
            if !value.is_finite() || value < 0.0 {
                return Err(format!(
                    "{name}: informe um valor finito maior ou igual a zero."
                ));
            }
        }
        let positive = [
            ("L1 aditivo", self.l1_sum_mh),
            ("L1 oposto", self.l1_opp_uh),
            ("Q de L1", self.l1_q),
            ("Frequência de Q", self.l1_q_hz),
            ("|Z| da rede por fase", self.grid_phase_z_ohm),
            ("Frequência de referência da rede", self.grid_z_ref_hz),
            ("|Z| da rede CM", self.grid_cm_z_ohm),
            ("R da fonte por fase", self.source_phase_r_ohm),
            ("R da fonte CM", self.source_cm_r_ohm),
            ("Frequência do ruído", self.noise_frequency_hz),
            ("Amplitude do ruído", self.noise_peak_v),
            ("Limite harmônico", self.noise_max_hz),
            ("Início da varredura", self.sweep_min_hz),
            ("Fim da varredura", self.sweep_max_hz),
            ("Frequência da alimentação", self.mains_frequency_hz),
        ];
        for (name, value) in positive {
            if !value.is_finite() || value <= 0.0 {
                return Err(format!("{name}: informe um valor finito maior que zero."));
            }
        }
        for (name, value) in [
            ("Atraso do ruído", self.noise_delay_deg),
            ("Fase inicial do ruído", self.noise_phase_a_deg),
            ("Atraso da alimentação", self.mains_phase_deg),
        ] {
            if !value.is_finite() {
                return Err(format!("{name}: o ângulo precisa ser finito."));
            }
        }
        if self.l1_sum_mh * 1e-3 <= self.l1_opp_uh * 1e-6 {
            return Err("L1 aditivo precisa ser maior que L1 oposto (verifique mH e µH).".into());
        }
        if self.grid_phase_r_ohm > self.grid_phase_z_ohm || self.grid_cm_r_ohm > self.grid_cm_z_ohm
        {
            return Err("R da rede não pode superar |Z| na frequência de referência.".into());
        }
        if self.l2_uh == 0.0 && self.l2_rs_ohm == 0.0 {
            return Err("L2 e sua resistência não podem ser ambos zero; curto ideal não suportado nesse ramo.".into());
        }
        if self.l3_uh == 0.0 && self.l3_rs_ohm == 0.0 {
            return Err("L3 e sua resistência não podem ser ambos zero; curto ideal não suportado nesse ramo.".into());
        }
        if self.r1_kohm + self.r2_kohm <= 0.0 {
            return Err("A soma R1 + R2 precisa ser maior que zero.".into());
        }
        if self.sweep_max_hz <= self.sweep_min_hz {
            return Err("O fim da varredura precisa superar o início.".into());
        }
        if !(2..=50000).contains(&self.sweep_points) {
            return Err("Use entre 2 e 50000 pontos de varredura.".into());
        }
        if self.noise_max_hz < self.noise_frequency_hz {
            return Err("O limite harmônico precisa incluir a fundamental do ruído.".into());
        }
        if self.noise_max_hz / self.noise_frequency_hz > 100001.0 {
            return Err("A série está longa demais: limite a razão fmax/f0 a 100001.".into());
        }
        if self.load_peak_a < self.load_rms_a {
            return Err("A corrente de pico não pode ser menor que a corrente RMS.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct CurvePoint {
    pub frequency_hz: f64,
    pub dm_db: f64,
    pub cm_db: f64,
}

#[derive(Clone, Debug)]
pub struct HarmonicPoint {
    pub order: usize,
    pub frequency_hz: f64,
    pub combined_db: f64,
    pub dm_db: Option<f64>,
    pub cm_db: Option<f64>,
    pub phase_a_db: f64,
    pub phase_b_db: f64,
}

#[derive(Clone, Debug)]
pub struct IntegratedLoss {
    pub order: usize,
    pub combined_db: f64,
    pub dm_db: f64,
    pub cm_db: f64,
    pub phase_a_db: f64,
    pub phase_b_db: f64,
}

#[derive(Clone, Debug)]
pub struct Simulation {
    pub points: Vec<CurvePoint>,
    pub harmonics: Vec<HarmonicPoint>,
    pub integrated: Vec<IntegratedLoss>,
}

/// Vdm=Va−Vb, Vcm=(Va+Vb)/2, Idm=(Ia−Ib)/2 e Icm=Ia+Ib.
/// Zdm e Zcm são impedâncias das portas modais completas.
fn modal(dm: C, cm: C) -> M2 {
    [
        [cm + dm / 4.0, cm - dm / 4.0],
        [cm - dm / 4.0, cm + dm / 4.0],
    ]
}

fn inv(a: M2) -> Result<M2, String> {
    let d = a[0][0] * a[1][1] - a[0][1] * a[1][0];
    if d.norm() < 1e-30 || !d.re.is_finite() || !d.im.is_finite() {
        return Err("Matriz de impedância singular ou fora da faixa numérica.".into());
    }
    Ok([[a[1][1] / d, -a[0][1] / d], [-a[1][0] / d, a[0][0] / d]])
}

fn mul(a: M2, b: V2) -> V2 {
    [
        a[0][0] * b[0] + a[0][1] * b[1],
        a[1][0] * b[0] + a[1][1] * b[1],
    ]
}

struct Nodal {
    n: usize,
    a: [[C; 7]; 7],
    b: [C; 7],
}

// Índices explícitos preservam a correspondência entre nós e linhas da matriz.
#[allow(clippy::needless_range_loop)]
impl Nodal {
    fn new() -> Self {
        Self {
            n: 6,
            a: [[ZERO; 7]; 7],
            b: [ZERO; 7],
        }
    }

    fn branch(&mut self, i: usize, j: Option<usize>, y: C) {
        self.a[i][i] += y;
        if let Some(j) = j {
            self.a[j][j] += y;
            self.a[i][j] -= y;
            self.a[j][i] -= y;
        }
    }

    fn port(&mut self, first: usize, y: M2, e: V2) {
        let rhs = mul(y, e);
        for i in 0..2 {
            self.b[first + i] += rhs[i];
            for j in 0..2 {
                self.a[first + i][first + j] += y[i][j];
            }
        }
    }

    fn pair(&mut self, first: usize, last: usize, y: M2) {
        for i in 0..2 {
            for j in 0..2 {
                self.a[first + i][first + j] += y[i][j];
                self.a[last + i][last + j] += y[i][j];
                self.a[first + i][last + j] -= y[i][j];
                self.a[last + i][first + j] -= y[i][j];
            }
        }
    }

    fn solve(&self) -> Result<[C; 7], String> {
        let mut m = self.a;
        let mut v = self.b;
        for k in 0..self.n {
            let mut pivot = k;
            for i in k + 1..self.n {
                if m[i][k].norm_sqr() > m[pivot][k].norm_sqr() {
                    pivot = i;
                }
            }
            if m[pivot][k].norm() < 1e-25 {
                return Err("Sistema nodal singular: revise os valores dos componentes.".into());
            }
            m.swap(k, pivot);
            v.swap(k, pivot);
            for i in k + 1..self.n {
                let q = m[i][k] / m[k][k];
                for j in k..self.n {
                    let value = m[k][j];
                    m[i][j] -= q * value;
                }
                let value = v[k];
                v[i] -= q * value;
            }
        }
        let mut x = [ZERO; 7];
        for i in (0..self.n).rev() {
            let mut value = v[i];
            for (j, xj) in x.iter().enumerate().take(self.n).skip(i + 1) {
                value -= m[i][j] * xj;
            }
            x[i] = value / m[i][i];
            if !x[i].re.is_finite() || !x[i].im.is_finite() {
                return Err("Resultado não finito: parâmetros fora da faixa numérica.".into());
            }
        }
        let mut residual: f64 = 0.0;
        let mut scale: f64 = 0.0;
        for i in 0..self.n {
            let mut sum = ZERO;
            let mut row = self.b[i].norm();
            for (j, xj) in x.iter().enumerate().take(self.n) {
                let term = self.a[i][j] * xj;
                sum += term;
                row += term.norm();
            }
            residual = residual.max((sum - self.b[i]).norm());
            scale = scale.max(row);
        }
        if residual > 1e-10 * scale.max(1e-15) {
            return Err("Resíduo nodal excessivo: resultado numericamente instável.".into());
        }
        Ok(x)
    }
}

fn cap_z(f: f64, capacitance_f: f64, esr: f64, esl_h: f64) -> C {
    let w = 2.0 * PI * f;
    C::new(esr, w * esl_h - 1.0 / (w * capacitance_f))
}

fn cap_y(f: f64, capacitance_f: f64, esr: f64, esl_h: f64, extra_r: f64) -> Result<C, String> {
    if capacitance_f == 0.0 {
        return Ok(ZERO);
    }
    let z = cap_z(f, capacitance_f, esr + extra_r, esl_h);
    if z.norm() < 1e-25 {
        return Err(
            "Capacitor/ESL em ressonância ideal sem ESR: informe uma perda não nula.".into(),
        );
    }
    Ok(ONE / z)
}

fn choke(p: &Parameters, f: f64, cm: bool) -> C {
    let (l, r) = if cm {
        let l = p.l1_sum_mh * 1e-3 / 4.0;
        (l, 2.0 * PI * p.l1_q_hz * l / p.l1_q)
    } else {
        (p.l1_opp_uh * 1e-6, 2.0 * p.l1_dcr_mohm * 1e-3)
    };
    C::new(r, 2.0 * PI * f * l)
}

#[derive(Clone, Copy)]
struct Response {
    dm: C,
    cm: C,
}

fn response(p: &Parameters, f: f64, excitation: V2) -> Result<Response, String> {
    let mut net = Nodal::new();
    net.port(
        0,
        inv(modal(p.grid_dm_impedance(f), p.grid_cm_impedance(f)))?,
        [ZERO, ZERO],
    );
    net.port(
        4,
        inv(modal(
            C::new(2.0 * p.source_phase_r_ohm, 0.0),
            C::new(p.source_cm_r_ohm, 0.0),
        ))?,
        excitation,
    );
    net.pair(0, 2, inv(modal(choke(p, f, false), choke(p, f, true)))?);

    // Os dois indutores de saída são independentes: a matriz diagonal admite assimetria.
    let z2 = C::new(p.l2_rs_ohm, 2.0 * PI * f * p.l2_uh * 1e-6);
    let z3 = C::new(p.l3_rs_ohm, 2.0 * PI * f * p.l3_uh * 1e-6);
    net.pair(2, 4, [[ONE / z2, ZERO], [ZERO, ONE / z3]]);
    if p.choke_cp_pf > 0.0 {
        let y = C::new(0.0, 2.0 * PI * f * p.choke_cp_pf * 1e-12);
        net.branch(0, Some(2), y);
        net.branch(1, Some(3), y);
    }
    net.branch(
        0,
        Some(1),
        cap_y(f, p.cx1_uf * 1e-6, p.cx1_esr_ohm, p.cx1_esl_nh * 1e-9, 0.0)?
            + ONE / ((p.r1_kohm + p.r2_kohm) * 1000.0),
    );
    net.branch(
        2,
        Some(3),
        cap_y(
            f,
            p.cx2_uf * 1e-6,
            p.cx2_esr_ohm,
            p.cx2_esl_nh * 1e-9,
            p.rd_ohm,
        )?,
    );
    net.branch(
        4,
        Some(5),
        cap_y(f, p.cx3_uf * 1e-6, p.cx3_esr_ohm, p.cx3_esl_nh * 1e-9, 0.0)?,
    );

    let zpe = C::new(p.pe_r_ohm, 2.0 * PI * f * p.pe_l_uh * 1e-6);
    let pe = if zpe.norm() > 0.0 {
        net.n = 7;
        net.branch(6, None, ONE / zpe);
        Some(6)
    } else {
        None
    };
    net.branch(
        2,
        pe,
        cap_y(f, p.cy1_nf * 1e-9, p.cy_esr_ohm, p.cy_esl_nh * 1e-9, 0.0)?,
    );
    net.branch(
        3,
        pe,
        cap_y(f, p.cy2_nf * 1e-9, p.cy_esr_ohm, p.cy_esl_nh * 1e-9, 0.0)?,
    );
    let v = net.solve()?;
    Ok(Response {
        dm: v[0] - v[1],
        cm: (v[0] + v[1]) / 2.0,
    })
}

fn unfiltered(p: &Parameters, f: f64, excitation: V2) -> Response {
    let ed = excitation[0] - excitation[1];
    let ec = (excitation[0] + excitation[1]) / 2.0;
    let zg = p.grid_dm_impedance(f);
    let zgcm = p.grid_cm_impedance(f);
    Response {
        dm: ed * zg / (2.0 * p.source_phase_r_ohm + zg),
        cm: ec * zgcm / (p.source_cm_r_ohm + zgcm),
    }
}

fn insertion(before_squared: f64, after_squared: f64) -> f64 {
    // Casos degenerados só ocorrem se o usuário remove um modo da excitação.
    // NaN é representado por None nos pontos harmônicos sem aquele modo.
    if before_squared <= 0.0 || after_squared <= 0.0 {
        return f64::NAN;
    }
    10.0 * (before_squared / after_squared).log10()
}

/// Norma conjunta, DM, CM, fase A e fase B; em pico² (fator RMS 1/2 cancela).
fn powers(r: Response) -> [f64; 5] {
    let a = r.cm + r.dm / 2.0;
    let b = r.cm - r.dm / 2.0;
    [
        a.norm_sqr() + b.norm_sqr(),
        r.dm.norm_sqr(),
        r.cm.norm_sqr(),
        a.norm_sqr(),
        b.norm_sqr(),
    ]
}

/// Perda de inserção comparada à mesma rede/fontes sem filtro. Valores negativos
/// representam reforço de tensão, e não ganho de potência numa rede passiva.
pub fn calculate(p: &Parameters) -> Result<Simulation, String> {
    p.validate()?;
    let mut frequencies = Vec::with_capacity(p.sweep_points + 16);
    let ratio = p.sweep_max_hz / p.sweep_min_hz;
    for i in 0..p.sweep_points {
        frequencies.push(p.sweep_min_hz * ratio.powf(i as f64 / (p.sweep_points - 1) as f64));
    }
    // Marcadores exatos para comparação com a ferramenta C++ e os gráficos gravados.
    for f in [
        60.0, 1000.0, 5000.0, 6000.0, 10000.0, 20000.0, 50000.0, 72000.0, 100000.0, 300000.0,
    ] {
        if f >= p.sweep_min_hz && f <= p.sweep_max_hz {
            frequencies.push(f);
        }
    }
    let max_order = (p.noise_max_hz / p.noise_frequency_hz * (1.0 + 1e-12)).floor() as usize;
    for n in (1..=max_order).step_by(2) {
        let f = n as f64 * p.noise_frequency_hz;
        if f >= p.sweep_min_hz && f <= p.sweep_max_hz {
            frequencies.push(f);
        }
    }
    frequencies.sort_by(f64::total_cmp);
    frequencies.dedup_by(|a, b| (*a - *b).abs() <= 1e-12 * a.abs().max(b.abs()));
    let mut points = Vec::with_capacity(frequencies.len());
    for f in frequencies {
        let excitation_dm = [C::new(0.5, 0.0), C::new(-0.5, 0.0)];
        let excitation_cm = [ONE, ONE];
        let dm = response(p, f, excitation_dm).map_err(|e| format!("{f:.3} Hz: {e}"))?;
        let cm = response(p, f, excitation_cm).map_err(|e| format!("{f:.3} Hz: {e}"))?;
        let dm0 = unfiltered(p, f, excitation_dm);
        let cm0 = unfiltered(p, f, excitation_cm);
        points.push(CurvePoint {
            frequency_hz: f,
            dm_db: insertion(dm0.dm.norm_sqr(), dm.dm.norm_sqr()),
            cm_db: insertion(cm0.cm.norm_sqr(), cm.cm.norm_sqr()),
        });
    }

    let mut harmonics = Vec::with_capacity(max_order.div_ceil(2));
    let mut before = [[0.0; 5]; 3];
    let mut after = [[0.0; 5]; 3];
    for n in (1..=max_order).step_by(2) {
        let f = n as f64 * p.noise_frequency_hz;
        let amplitude = 4.0 * p.noise_peak_v / (PI * n as f64);
        // Um atraso temporal: o ângulo relativo também é multiplicado por n.
        let ea = C::new(0.0, -amplitude)
            * C::from_polar(1.0, n as f64 * p.noise_phase_a_deg.to_radians());
        let eb = C::new(0.0, -amplitude)
            * C::from_polar(
                1.0,
                n as f64 * (p.noise_phase_a_deg - p.noise_delay_deg).to_radians(),
            );
        let filtered =
            response(p, f, [ea, eb]).map_err(|e| format!("Harmônica {n} ({f:.3} Hz): {e}"))?;
        let baseline = unfiltered(p, f, [ea, eb]);
        let pb = powers(baseline);
        let pa = powers(filtered);
        let mode_is_present = |input: C| input.norm() > 1e-12 * amplitude;
        harmonics.push(HarmonicPoint {
            order: n,
            frequency_hz: f,
            combined_db: insertion(pb[0], pa[0]),
            dm_db: mode_is_present(ea - eb).then(|| insertion(pb[1], pa[1])),
            cm_db: mode_is_present((ea + eb) / 2.0).then(|| insertion(pb[2], pa[2])),
            phase_a_db: insertion(pb[3], pa[3]),
            phase_b_db: insertion(pb[4], pa[4]),
        });
        // d^m/dt^m multiplica o fasor por (jω)^m: a norma ganha ω^(2m).
        let omega2 = (2.0 * PI * f).powi(2);
        let mut factor = 0.5;
        for order in 0..3 {
            for j in 0..5 {
                before[order][j] += pb[j] * factor;
                after[order][j] += pa[j] * factor;
            }
            factor *= omega2;
        }
    }
    let dm_present = harmonics.iter().any(|h| h.dm_db.is_some());
    let cm_present = harmonics.iter().any(|h| h.cm_db.is_some());
    let integrated = (0..3)
        .map(|order| IntegratedLoss {
            order,
            combined_db: insertion(before[order][0], after[order][0]),
            dm_db: if dm_present {
                insertion(before[order][1], after[order][1])
            } else {
                f64::NAN
            },
            cm_db: if cm_present {
                insertion(before[order][2], after[order][2])
            } else {
                f64::NAN
            },
            phase_a_db: insertion(before[order][3], after[order][3]),
            phase_b_db: insertion(before[order][4], after[order][4]),
        })
        .collect();
    Ok(Simulation {
        points,
        harmonics,
        integrated,
    })
}

/// CSV único com tipos distintos de linha e referência explícita de perda de inserção.
pub fn export_csv(simulation: &Simulation) -> String {
    use std::fmt::Write;
    let mut output = String::from("tipo,frequencia_Hz,harmonica,ordem_derivada,IL_DM_dB,IL_CM_dB,IL_conjunta_dB,IL_fase_A_dB,IL_fase_B_dB\n");
    for p in &simulation.points {
        let _ = writeln!(
            output,
            "curva,{:.12},,,{:.12},{:.12},,,",
            p.frequency_hz, p.dm_db, p.cm_db
        );
    }
    let opt = |value: Option<f64>| {
        value
            .filter(|v| v.is_finite())
            .map(|v| format!("{v:.12}"))
            .unwrap_or_default()
    };
    for h in &simulation.harmonics {
        let _ = writeln!(
            output,
            "harmonica,{:.12},{},,{},{},{:.12},{:.12},{:.12}",
            h.frequency_hz,
            h.order,
            opt(h.dm_db),
            opt(h.cm_db),
            h.combined_db,
            h.phase_a_db,
            h.phase_b_db
        );
    }
    for r in &simulation.integrated {
        let _ = writeln!(
            output,
            "RMS,,,{},{},{},{},{},{}",
            r.order,
            opt(Some(r.dm_db)),
            opt(Some(r.cm_db)),
            opt(Some(r.combined_db)),
            opt(Some(r.phase_a_db)),
            opt(Some(r.phase_b_db))
        );
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tolerance: f64) {
        assert!(
            (a - b).abs() <= tolerance,
            "{a:.12} != {b:.12} (tol={tolerance})"
        );
    }

    fn product(a: M2, b: M2) -> M2 {
        let mut r = [[ZERO; 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                for (k, _) in b.iter().enumerate() {
                    r[i][j] += a[i][k] * b[k][j];
                }
            }
        }
        r
    }

    fn series(z: C) -> M2 {
        [[ONE, z], [ZERO, ONE]]
    }
    fn shunt(y: C) -> M2 {
        [[ONE, ZERO], [y, ONE]]
    }

    /// Cálculo escalar ABCD independente para o caso simétrico (sem Cp).
    fn abcd(p: &Parameters, f: f64, cm: bool) -> C {
        let y1 = if cm {
            ZERO
        } else {
            cap_y(f, p.cx1_uf * 1e-6, p.cx1_esr_ohm, p.cx1_esl_nh * 1e-9, 0.0).unwrap()
                + ONE / ((p.r1_kohm + p.r2_kohm) * 1000.0)
        };
        let zcy = cap_z(f, p.cy1_nf * 1e-9, p.cy_esr_ohm, p.cy_esl_nh * 1e-9);
        let zpe = C::new(p.pe_r_ohm, 2.0 * PI * f * p.pe_l_uh * 1e-6);
        let ym = if cm {
            2.0 / (zcy + 2.0 * zpe)
        } else {
            ONE / (2.0 * zcy)
                + cap_y(
                    f,
                    p.cx2_uf * 1e-6,
                    p.cx2_esr_ohm,
                    p.cx2_esl_nh * 1e-9,
                    p.rd_ohm,
                )
                .unwrap()
        };
        let yo = if cm {
            ZERO
        } else {
            cap_y(f, p.cx3_uf * 1e-6, p.cx3_esr_ohm, p.cx3_esl_nh * 1e-9, 0.0).unwrap()
        };
        let zadd = C::new(p.l2_rs_ohm, 2.0 * PI * f * p.l2_uh * 1e-6) * if cm { 0.5 } else { 2.0 };
        let mut t = product(shunt(y1), series(choke(p, f, cm)));
        t = product(t, shunt(ym));
        t = product(t, series(zadd));
        t = product(t, shunt(yo));
        let (zg, rs) = if cm {
            (p.grid_cm_impedance(f), p.source_cm_r_ohm)
        } else {
            (p.grid_dm_impedance(f), 2.0 * p.source_phase_r_ohm)
        };
        zg / (t[1][1] * zg + t[0][1] + rs * (t[1][0] * zg + t[0][0]))
    }

    #[test]
    fn defaults_match_saved_cpp_model() {
        let p = Parameters::default();
        let s = calculate(&p).unwrap();
        let expected = [
            [11.36742799, 28.27482694, -0.45962463],
            [11.08249387, 34.69871394, 7.06191894],
            [19.11438005, 47.28962425, 16.43291829],
        ];
        for (r, expected) in s.integrated.iter().zip(expected) {
            close(r.combined_db, expected[0], 1e-7);
            close(r.dm_db, expected[1], 1e-7);
            close(r.cm_db, expected[2], 1e-7);
        }
        let f10 = s.points.iter().find(|p| p.frequency_hz == 10000.0).unwrap();
        close(f10.dm_db, 21.34955763, 1e-7);
        close(s.harmonics[0].combined_db, 18.19424145, 1e-7);
        close(s.harmonics[1].combined_db, -3.20628696, 1e-7);
        assert!(s.harmonics[1].dm_db.is_none());
    }

    #[test]
    fn nodal_matches_independent_abcd_with_and_without_pe_impedance() {
        for pe in [false, true] {
            let mut p = Parameters::default();
            if pe {
                p.pe_r_ohm = 0.4;
                p.pe_l_uh = 1.5;
            }
            for f in [
                60.0, 1000.0, 1980.8, 10000.0, 20000.0, 60000.0, 100000.0, 300000.0,
            ] {
                for cm in [false, true] {
                    let e = if cm {
                        [ONE, ONE]
                    } else {
                        [C::new(0.5, 0.0), C::new(-0.5, 0.0)]
                    };
                    let h = response(&p, f, e).unwrap();
                    let (measured, cross) = if cm { (h.cm, h.dm) } else { (h.dm, h.cm) };
                    assert!((measured - abcd(&p, f, cm)).norm() < 2e-9);
                    assert!(cross.norm() < 2e-9);
                }
            }
        }
    }

    #[test]
    fn grid_is_inductive_and_calibrated_per_phase() {
        let p = Parameters::default();
        close(p.grid_phase_impedance(60.0).norm(), 0.5, 1e-14);
        close(p.grid_dm_impedance(60.0).norm(), 1.0, 1e-14);
        close(p.grid_dm_impedance(10000.0).im, 10000.0 / 60.0, 1e-10);
        close(p.grid_cm_impedance(10000.0).im, 10000.0 / 120.0, 1e-10);
        close(p.line_voltage_rms(), 219.9704525612474, 1e-10);
        let mut lossy = p.clone();
        lossy.grid_phase_r_ohm = 0.3;
        close(lossy.grid_phase_impedance(60.0).norm(), 0.5, 1e-14);
        close(lossy.grid_phase_impedance(600.0).re, 0.3, 1e-14);
        close(lossy.grid_phase_impedance(600.0).im, 4.0, 1e-14);
    }

    #[test]
    fn absolute_carrier_phase_and_mains_phase_do_not_change_spectral_attenuation() {
        let mut a = Parameters::default();
        a.sweep_points = 20;
        let sa = calculate(&a).unwrap();
        a.noise_phase_a_deg = 91.0;
        a.mains_phase_deg = 73.0;
        let sb = calculate(&a).unwrap();
        for (ra, rb) in sa.integrated.iter().zip(&sb.integrated) {
            close(ra.combined_db, rb.combined_db, 1e-9);
        }
    }

    #[test]
    fn asymmetry_and_parasitics_are_solved_in_full_network() {
        let mut p = Parameters::default();
        p.sweep_points = 40;
        p.cy2_nf = 4.7;
        p.l3_uh = 1.8;
        p.l3_rs_ohm = 0.2;
        p.pe_r_ohm = 0.25;
        p.pe_l_uh = 2.0;
        p.choke_cp_pf = 30.0;
        p.cx1_esr_ohm = 0.02;
        p.cx1_esl_nh = 10.0;
        let s = calculate(&p).unwrap();
        assert!(s
            .points
            .iter()
            .all(|v| v.dm_db.is_finite() && v.cm_db.is_finite()));
        assert!(s.integrated.iter().all(|v| v.combined_db.is_finite()));
        let h = response(&p, 20000.0, [C::new(0.5, 0.0), C::new(-0.5, 0.0)]).unwrap();
        assert!(h.cm.norm() > 1e-8);
    }

    #[test]
    fn zero_capacitance_removes_branch_and_invalid_short_is_rejected() {
        let mut p = Parameters::default();
        p.sweep_points = 20;
        p.cx2_uf = 0.0;
        p.cy1_nf = 0.0;
        assert!(calculate(&p).is_ok());
        p.l2_uh = 0.0;
        p.l2_rs_ohm = 0.0;
        assert!(calculate(&p).unwrap_err().contains("L2"));
        p = Parameters::default();
        p.grid_phase_r_ohm = 0.6;
        assert!(calculate(&p).unwrap_err().contains("|Z|"));
    }

    #[test]
    fn absent_mode_has_no_spurious_rms_attenuation_or_csv_number() {
        for delay in [0.0, 180.0, 360.0] {
            let mut p = Parameters::default();
            p.sweep_points = 20;
            p.noise_delay_deg = delay;
            let s = calculate(&p).unwrap();
            for r in &s.integrated {
                assert!(r.combined_db.is_finite());
                if delay == 180.0 {
                    assert!(r.cm_db.is_nan());
                    assert!(r.dm_db.is_finite());
                } else {
                    assert!(r.dm_db.is_nan());
                    assert!(r.cm_db.is_finite());
                }
            }
            let csv = export_csv(&s);
            assert!(!csv.contains("NaN"));
            assert!(csv.lines().all(|row| row.split(',').count() == 9));
        }
    }
}
