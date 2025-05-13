pub enum Param {
    Input(f32),
    Output(f32),
    Pan(f32),
    Feedback(f32),
    Time(usize),
    Clear,
}

pub struct LoopLine {
    read_pos: usize,
    write_pos: usize,
    buffer: Vec<f32>,
    pub in_vol: f32,
    pub out_vol: f32,
    pub len: usize,
    pub feedback: f32,
}

impl LoopLine {
    pub fn new(length: usize, time: usize) -> Self {
        let len = length;
        Self {
            buffer: vec![0.0; len],
            write_pos: 0,
            read_pos: 0,
            len: time,
            in_vol: 0.0,
            out_vol: 0.0,
            feedback: 0.0,
        }
    }

    pub fn write(&mut self, value: f32) {
        self.buffer[self.write_pos] =
            value * self.in_vol + self.buffer[self.write_pos] * self.feedback;
        self.write_pos = (self.write_pos + 1) % self.len;
    }

    pub fn write_slice(&mut self, values: &[f32]) {
        for value in values {
            self.write(*value);
        }
    }

    pub fn read_advance(&mut self, samples: u32) -> f32 {
        let out = self.buffer[self.read_pos];
        self.read_pos = (self.read_pos + (samples as usize)) % self.len;
        out * self.out_vol
    }

    pub fn read_slice(&mut self, buffer: &mut [f32], skip: u32) {
        for x in buffer.iter_mut() {
            *x = self.read_advance(skip);
        }
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.len = self.buffer.len();
        self.write_pos = 0;
        self.read_pos = 0;
    }
}

pub struct StereoLine {
    pub l_line: LoopLine,
    pub r_line: LoopLine,
    pub pan: f32,
    pub out_vol: f32,
}

impl StereoLine {
    pub fn new(length: usize) -> Self {
        Self {
            l_line: LoopLine::new(length, length),
            r_line: LoopLine::new(length, length),
            pan: 0.0,
            out_vol: 0.0,
        }
    }

    pub fn write(&mut self, v_l: f32, v_r: f32) {
        self.l_line.write(v_l);
        self.r_line.write(v_r);
    }

    pub fn write_slice(&mut self, v_l: &[f32], v_r: &[f32]) {
        self.l_line.write_slice(v_l);
        self.r_line.write_slice(v_r);
    }

    pub fn read_advance(&mut self, samples: u32) -> (f32, f32) {
        (
            self.l_line.read_advance(samples),
            self.r_line.read_advance(samples),
        )
    }

    pub fn read_slice(&mut self, buffers: (&mut [f32], &mut [f32]), skip: u32) {
        self.l_line.read_slice(buffers.0, skip);
        self.r_line.read_slice(buffers.1, skip);
    }

    fn calc_pan(&mut self) {
        let pan_mult = self.pan / 2.0 + 0.5;
        self.r_line.out_vol = pan_mult * self.out_vol;
        let pan_mult = 1.0 - pan_mult;
        self.l_line.out_vol = pan_mult * self.out_vol;
    }

    pub fn send_param(&mut self, param: Param) {
        match param {
            Param::Feedback(x) => {
                self.l_line.feedback = x;
                self.r_line.feedback = x;
            }
            Param::Time(x) => {
                self.l_line.len = x;
                self.r_line.len = x;
            }
            Param::Clear => {
                self.l_line.clear();
                self.r_line.clear();
            }
            Param::Input(x) => {
                self.l_line.in_vol = x;
                self.r_line.in_vol = x;
            }
            Param::Output(x) => {
                self.out_vol = x;
                self.calc_pan();
            }
            Param::Pan(x) => {
                self.pan = x;
                self.calc_pan();
            }
        }
    }
}
