use super::opcode::Register;

pub struct Closure {
    arg_len: usize,
    reg_len: usize,
    has_var_args: bool,
    late_binding: bool,
}

impl Closure {
    pub fn new(late_binding: bool) -> Self {
        Self {
            late_binding,
            arg_len: 0,
            has_var_args: false,
            reg_len: 0,
        }
    }

    pub fn late_binding(&self) -> bool {
        self.late_binding
    }

    pub fn arg_len(&self) -> usize {
        self.arg_len
    }
    pub fn has_var_args(&self) -> bool {
        self.has_var_args
    }
    pub fn reg_len(&self) -> usize {
        self.reg_len
    }

    pub fn add_arg(&mut self) -> Register {
        self.arg_len += 1;
        self.add_reg()
    }

    pub fn add_var_arg(&mut self) -> Register {
        self.has_var_args = true;
        self.add_reg()
    }

    pub fn add_reg(&mut self) -> Register {
        self.reg_len += 1;
        Register::new(self.reg_len)
    }
}
