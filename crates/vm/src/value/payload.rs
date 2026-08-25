const PAYLOAD_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

/// A six-byte payload.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub(crate) struct Payload([u8; 6]);

impl Payload {
    pub const MASK: u64 = PAYLOAD_MASK;

    pub const fn to_bytes(self) -> [u8; 6] {
        self.0
    }

    pub const fn from_bytes(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    pub const fn from_bits(bits: u64) -> Self {
        debug_assert!(bits <= PAYLOAD_MASK, "Payload bits must be 48 bits or less");
        let bytes = bits.to_le_bytes();
        Self([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]])
    }

    pub const fn to_bits(self) -> u64 {
        let bytes = [
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], 0, 0,
        ];
        u64::from_le_bytes(bytes)
    }

    #[inline]
    pub fn from_address<T>(address: *const T) -> Self {
        let address = address.expose_provenance() as u64;
        assert!(
            address <= PAYLOAD_MASK,
            "Pointer too large to store in MiraValue"
        );
        Self::from_bits(address)
    }

    #[inline]
    pub fn to_address<T>(self) -> *const T {
        let address = self.to_bits();
        std::ptr::with_exposed_provenance::<T>(address as usize)
    }
}
