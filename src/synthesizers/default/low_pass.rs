fn tanhxdx(x: f64) -> f64 {
    // x.tanh() / x
    let a = x * x;
    ((a + 105.0) * a + 945.0) / ((15.0 * a + 420.0) * a + 945.0)
}

//// LICENSE TERMS: Copyright 2012 Teemu Voipio
//
// You can use this however you like for pretty much any purpose,
// as long as you don't claim you wrote it. There is no warranty.
//
// Distribution of substantial portions of this code in source form
// must include this copyright notice and list of conditions.
//
// From https://www.kvraudio.com/forum/viewtopic.php?f=33&t=349859.
pub struct MystramFilter {
    sample_rate: f64,
    zi: f64,
    s: [f64; 4],
    f: f64,
    r: f64,
}

impl MystramFilter {
    pub fn new(sample_rate: f64) -> Self {
        let mut filter = Self {
            sample_rate,
            zi: 0.0,
            s: [0.0; 4],
            f: 0.0,
            r: 0.0,
        };

        filter.set_cutoff(1000.0);
        filter.set_resonance(0.5);
        filter
    }

    pub fn set_cutoff(&mut self, cutoff: f64) {
        self.f = (cutoff / self.sample_rate * std::f64::consts::PI).tan();
    }

    pub fn set_resonance(&mut self, resonance: f64) {
        self.r = 40.0 / 9.0 * resonance;
    }

    pub fn process(&mut self, sample: f64) -> f64 {
        // Input with half delay, for non-linearities.
        let ih = 0.5 * (sample + self.zi);
        self.zi = sample;

        // Evaluate the non-linear gains.
        let t0 = tanhxdx(ih - self.r * self.s[3]);
        let t1 = tanhxdx(self.s[0]);
        let t2 = tanhxdx(self.s[1]);
        let t3 = tanhxdx(self.s[2]);
        let t4 = tanhxdx(self.s[3]);

        // G# the denominators for solutions of individual stages.
        let g0 = 1.0 / (1.0 + self.f * t1);
        let g1 = 1.0 / (1.0 + self.f * t2);
        let g2 = 1.0 / (1.0 + self.f * t3);
        let g3 = 1.0 / (1.0 + self.f * t4);

        // F# are just factored out of the feedback solution.
        let f3 = self.f * t3 * g3;
        let f2 = self.f * t2 * g2 * f3;
        let f1 = self.f * t1 * g1 * f2;
        let f0 = self.f * t0 * g0 * f1;

        // Solve feedback.
        let y3 = (g3 * self.s[3]
            + f3 * g2 * self.s[2]
            + f2 * g1 * self.s[1]
            + f1 * g0 * self.s[0]
            + f0 * sample)
            / (1.0 + self.r * f0);

        // Then solve the remaining outputs (with the non-linear gains here).
        let xx = t0 * (sample - self.r * y3);
        let y0 = t1 * g0 * (self.s[0] + self.f * xx);
        let y1 = t2 * g1 * (self.s[1] + self.f * y0);
        let y2 = t3 * g2 * (self.s[2] + self.f * y1);

        // update state
        self.s[0] += 2.0 * self.f * (xx - y0);
        self.s[1] += 2.0 * self.f * (y0 - y1);
        self.s[2] += 2.0 * self.f * (y1 - y2);
        self.s[3] += 2.0 * self.f * (y2 - t4 * y3);

        y3
    }
}
