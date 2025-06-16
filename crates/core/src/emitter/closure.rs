use super::opcode::Register;

pub struct Closure {
    reg_len: usize,
    late_binding: bool,
}

impl Closure {
    /// 创建新的闭包，arg_len 是参数的数量，会预先分配相同数量的寄存器
    pub fn new(late_binding: bool, arg_len: usize) -> Self {
        Self {
            late_binding,
            reg_len: arg_len,
        }
    }

    pub fn late_binding(&self) -> bool {
        self.late_binding
    }
    pub fn reg_len(&self) -> usize {
        self.reg_len
    }
    pub fn add_reg(&mut self) -> Register {
        self.reg_len += 1;
        Register::new(self.reg_len)
    }
}
