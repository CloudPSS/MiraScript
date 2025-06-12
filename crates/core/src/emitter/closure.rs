use super::opcode::Register;

pub struct Closure {
    arg_len: usize,
    reg_len: usize,
    has_var_args: bool,
    late_binding: bool,
}

impl Closure {
    pub fn new(late_binding: bool, arg_len: usize) -> Self {
        Self {
            late_binding,
            arg_len,
            has_var_args: false,
            reg_len: arg_len,
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
    pub fn set_var_args(&mut self) {
        self.has_var_args = true;
    }

    pub fn add_reg(&mut self) -> Register {
        self.reg_len += 1;
        Register::new(self.reg_len)
    }
}
