use std::io::Read;

use byteorder::{
    ByteOrder,
    ReadBytesExt,
};

pub trait ReadVecEx {
    fn read_vec3_f32<B: ByteOrder>(&mut self) -> std::io::Result<nalgebra::Vector3<f32>>;
    fn read_vec3_u32<B: ByteOrder>(&mut self) -> std::io::Result<nalgebra::Vector3<u32>>;
}

impl<T> ReadVecEx for T
where
    T: Read,
{
    fn read_vec3_f32<B: ByteOrder>(&mut self) -> std::io::Result<nalgebra::Vector3<f32>> {
        let x = self.read_f32::<B>()?;
        let y = self.read_f32::<B>()?;
        let z = self.read_f32::<B>()?;
        Ok(nalgebra::Vector3::new(x, y, z))
    }

    fn read_vec3_u32<B: ByteOrder>(&mut self) -> std::io::Result<nalgebra::Vector3<u32>> {
        let x = self.read_u32::<B>()?;
        let y = self.read_u32::<B>()?;
        let z = self.read_u32::<B>()?;
        Ok(nalgebra::Vector3::new(x, y, z))
    }
}
